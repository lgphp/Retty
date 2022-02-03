use std::borrow::Borrow;
use std::io::ErrorKind;
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
    pub(crate) channel_map: Arc<CHashMap<Token, Arc<Mutex<Channel>>>>,
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

    pub(crate) fn attach(&self, id: usize, ch: Arc<Mutex<Channel>>, mut ctx__inbound_ctx_pipe: ChannelInboundHandlerCtxPipe) {
        let channel = ch.clone();
        let channel_2 = ch.clone();
        // 一个channel注册一个selector
        {
            let channel = channel.lock().unwrap();
            channel.register(&self.selector);
        }
        {
            ctx__inbound_ctx_pipe.head_channel_active();
        }
        self.channel_inbound_handler_ctx_pipe_map.insert_new(Token(id), ctx__inbound_ctx_pipe);

        self.channel_map.insert_new(Token(id), channel_2);
    }


    pub(crate) fn run(&self) {
        let selector = Arc::clone(&self.selector);
        let channel_map = Arc::clone(&self.channel_map);
        let channel_inbound_ctx_pipe_map = Arc::clone(&self.channel_inbound_handler_ctx_pipe_map);
        let stopped = Arc::clone(&self.stopped);

        self.excutor.spawn(move || {
            let mut events = Events::with_capacity(1024);
            while !stopped.load(Ordering::Relaxed) {
                selector.poll(&mut events, Some(Duration::from_millis(200))).unwrap();

                for e in events.iter() {
                    let channel = match channel_map.remove(&e.token()) {
                        Some(ch) => {
                            let mut buf: Vec<u8> = Vec::with_capacity(65535);
                            let ch_clone = ch.clone();
                            let mut ch = ch.lock().unwrap();
                            let ch_ret = match ch.read(&mut buf) {
                                Ok(0) => {
                                    ch.close();
                                    None
                                }
                                Ok(n) => {
                                    None
                                }
                                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                    None
                                }
                                Err(e) => {
                                    Some(e)
                                }
                            };
                            if !ch.is_closed() {
                                channel_map.insert_new(e.token(), ch_clone);
                            }
                            Some((ch.clone(), buf.clone(), ch_ret))
                        }
                        None => None
                    };
                    if let Some((ch, buf, err)) = channel {
                        if ch.is_closed() {
                            {
                                let ctx_pipe = channel_inbound_ctx_pipe_map.get_mut(&e.token()).unwrap();
                                ctx_pipe.head_channel_inactive();
                            }
                        }
                        if !ch.is_closed() {
                            if err.is_some() {
                                let ctx_pipe = channel_inbound_ctx_pipe_map.get_mut(&e.token()).unwrap();
                                let error: RettyErrorKind = err.unwrap().into();
                                ctx_pipe.head_channel_exception(error);
                            } else {
                                let mut bytebuf = ByteBuf::new_from(&buf[..]);
                                let ctx_pipe = channel_inbound_ctx_pipe_map.get_mut(&e.token()).unwrap();
                                ctx_pipe.head_channel_read(&mut bytebuf);
                            }
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
        if self.next > self.evenetloop_num - 1 {
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

