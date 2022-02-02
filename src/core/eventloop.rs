use std::borrow::Borrow;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::thread;
use std::thread::Thread;
use std::time::Duration;

use bytebuf_rs::bytebuf::ByteBuf;
use chashmap::CHashMap;
use mio::{Events, Poll, Token};
use rayon_core::ThreadPool;
use uuid::Uuid;

use crate::errors::RettyErrorKind;
use crate::handler::channel_handler_ctx_pipe::{ChannelInboundHandlerCtxPipe, ChannelOutboundHandlerCtxPipe};
use crate::transport::channel::{Channel, InboundChannelCtx, OutboundChannelCtx};

pub struct EventLoop {
    pub(crate) excutor: Arc<ThreadPool>,
    pub(crate) selector: Arc<Poll>,
    pub(crate) channel_map: Arc<CHashMap<Token, Channel>>,
    pub(crate) channel_inbound_handler_ctx_pipe_map: Arc<CHashMap<Token, ChannelInboundHandlerCtxPipe>>,
    pub(crate) stopped: Arc<AtomicBool>,
}


impl EventLoop {
    pub fn new() -> EventLoop {
        EventLoop {
            excutor: Arc::new(rayon::ThreadPoolBuilder::new().num_threads(1).thread_name(|mut n| {
                n = n + 1;
                format!("eventloop-{}", n)
            }).build().unwrap()),
            selector: Arc::new(Poll::new().unwrap()),
            channel_map: Arc::new(CHashMap::new()),
            channel_inbound_handler_ctx_pipe_map: Arc::new(CHashMap::new()),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn shutdown(&self) {
        self.stopped.store(true, Ordering::Relaxed);
    }

    pub(crate) fn attach(&self, id: usize, ch: Channel, mut ctx__inbound_ctx_pipe: ChannelInboundHandlerCtxPipe) {
        let mut channel = ch;
        // 一个channel注册一个selector
        channel.register(&self.selector);
        {
            ctx__inbound_ctx_pipe.head_channel_active();
        }
        self.channel_inbound_handler_ctx_pipe_map.insert_new(Token(id), ctx__inbound_ctx_pipe);

        self.channel_map.insert_new(Token(id), channel);
    }


    pub(crate) fn run(&self) {
        let selector = Arc::clone(&self.selector);
        let channel_map = Arc::clone(&self.channel_map);
        let channel_inbound_ctx_pipe_map = Arc::clone(&self.channel_inbound_handler_ctx_pipe_map);
        let stopped = Arc::clone(&self.stopped);

        self.excutor.spawn(move || {
            let mut events = Events::with_capacity(1024);
            while !stopped.load(Ordering::Relaxed) {
                selector.poll(&mut events, None).unwrap();

                for e in events.iter() {
                    if let Some(mut ch) = channel_map.remove(&e.token()) {
                        let mut buf: Vec<u8> = Vec::with_capacity(65535);
                        match ch.read(&mut buf) {
                            Ok(0) => {
                                ch.close();
                                {
                                    // 同步该channel所属pipeline 中的所有channel状态
                                    let inbound_ctx_pipe_guard = channel_inbound_ctx_pipe_map.get_mut(&e.token()).unwrap();
                                    for x in &inbound_ctx_pipe_guard.channel_handler_ctx_pipe {
                                        let mut ctx = x.lock().unwrap();
                                        {
                                            let out_bound = ctx.channel_ctx.channel.outbound_context_pipe.as_ref().unwrap();
                                            let out_bound_guard = out_bound.lock().unwrap();
                                            for outctx_mutex in &out_bound_guard.channel_handler_ctx_pipe {
                                                let mut out_ctx = outctx_mutex.lock().unwrap();
                                                out_ctx.channel_ctx = OutboundChannelCtx::new(ch.clone());
                                            }
                                        }
                                        ctx.channel_ctx = InboundChannelCtx::new(ch.clone());
                                    }
                                }
                                {
                                    let ctx_pipe = channel_inbound_ctx_pipe_map.get_mut(&e.token()).unwrap();
                                    ctx_pipe.head_channel_inactive();
                                }
                            }
                            Ok(n) => {}
                            Err(_) => {}
                        }
                        if !ch.is_closed() {
                            let mut bytebuf = ByteBuf::new_from(&buf[..]);
                            {
                                let ctx_pipe = channel_inbound_ctx_pipe_map.get_mut(&e.token()).unwrap();
                                ctx_pipe.head_channel_read(&mut bytebuf);
                            }
                        }
                        if !ch.is_closed() {
                            channel_map.insert_new(e.token(), ch);
                        }
                    }
                }
            }
        });
    }


    pub fn schedule_delayed<F>(&self, task: F, delay_ms: usize)
        where F: FnOnce() + Send + 'static {
        thread::sleep(Duration::from_millis(delay_ms as u64));
        self.excutor.spawn(task)
    }
}

pub struct EventLoopGroup {
    group: Vec<Arc<EventLoop>>,
    evenetloop_num: usize,
    next: usize,
}

impl EventLoopGroup {
    pub fn new(n: usize) -> EventLoopGroup {
        let mut _group = Vec::<Arc<EventLoop>>::new();
        for _i in 0..n {
            _group.push(Arc::new(EventLoop::new()));
        }
        EventLoopGroup {
            group: _group,
            evenetloop_num: n,
            next: 0,
        }
    }

    pub fn new_default_event_loop(n: usize) -> EventLoopGroup {
        EventLoopGroup::new(n)
    }

    pub fn next(&mut self) -> Option<Arc<EventLoop>> {
        self.next = self.next + 1;
        if self.next > self.evenetloop_num {
            self.next = 0;
        }
        Some(self.group.get(self.next).unwrap().clone())
    }

    pub fn execute<F>(&mut self, task: F) where F: FnOnce() + Send + 'static {
        let executor = self.next().unwrap();
        executor.excutor.spawn(task);
    }

    pub fn event_loop_group(&self) -> &Vec<Arc<EventLoop>> {
        &self.group
    }
}

