use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::thread;

use bytebuf_rs::bytebuf::ByteBuf;
use chashmap::CHashMap;
use mio::{Events, Poll, Token};
use rayon_core::ThreadPool;
use uuid::Uuid;

use crate::handler::channel_handler_ctx_pipe::{ChannelInboundHandlerCtxPipe, ChannelOutboundHandlerCtxPipe};
use crate::transport::channel::Channel;

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


    pub(crate) fn attach(&self, id: usize, ch: Channel, mut ctx__inbound_ctx_pipe: ChannelInboundHandlerCtxPipe) {
        let mut channel = ch;
        // 一个channel注册一个selector
        channel.register(&self.selector);
        {
            // 注册成功后，执行有新客户端连接上来的回调
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
                        let mut buf: Vec<u8> = Vec::with_capacity(65536);
                        let mut all_buf = Vec::<u8>::new();
                        match ch.read(&mut buf) {
                            Ok(0) => {
                                println!("close by remote peer.");
                                ch.close();
                            }
                            Ok(n) => {}
                            Err(e) => {}
                        }
                        if !ch.is_closed() {
                            let mut bytebuf = ByteBuf::new_from(&buf[..]);
                            println!("reactor-excutor :{}", thread::current().name().unwrap());
                            {
                                let ctx_pipe = channel_inbound_ctx_pipe_map.get_mut(&e.token()).unwrap();
                                ctx_pipe.head_channel_read(&bytebuf);
                            }
                        }

                        if !ch.is_closed() {
                            // 重置所有channnel
                            channel_map.insert_new(e.token(), ch);
                        }
                    }
                }
            }
        });
    }
}

pub struct EventLoopGroup {
    group: Vec<Arc<EventLoop>>,
}

impl EventLoopGroup {
    pub fn new(n: usize) -> EventLoopGroup {
        let mut _group = Vec::<Arc<EventLoop>>::new();
        for _i in 0..n {
            _group.push(Arc::new(EventLoop::new()));
        }
        EventLoopGroup {
            group: _group
        }
    }

    pub fn event_loop_group(&self) -> &Vec<Arc<EventLoop>> {
        &self.group
    }
}

