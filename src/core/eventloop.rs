use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::thread;

use bytebuf_rs::bytebuf::ByteBuf;
use chashmap::CHashMap;
use mio::{Events, Poll, Token};
use rayon_core::ThreadPool;
use uuid::Uuid;

use crate::handler::channel_handler_ctx_pipe::ChannelHandlerCtxPipe;
use crate::transport::channel::Channel;

pub struct EventLoop {
    pub(crate) excutor: Arc<ThreadPool>,
    pub(crate) selector: Arc<Poll>,
    pub(crate) channel_map: Arc<CHashMap<Token, Channel>>,
    pub(crate) channel_handler_ctx_pipe_map: Arc<CHashMap<Token, ChannelHandlerCtxPipe>>,
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
            channel_handler_ctx_pipe_map: Arc::new(CHashMap::new()),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }


    pub(crate) fn attach(&self, id: usize, ch: Channel, mut ctx_pipe: ChannelHandlerCtxPipe) {
        let mut channel = ch;
        // 一个channel注册一个selector
        channel.register(&self.selector);

        {
            // 注册成功后，执行有新客户端连接上来的回调
            let mut ctx_head = ctx_pipe.header_handler_ctx();
            let head_handler_clone = ctx_pipe.header_handler().clone();
            let head_handler = head_handler_clone.lock().unwrap();
            head_handler.channel_active(ctx_head);
        }

        self.channel_handler_ctx_pipe_map.insert_new(Token(id), ctx_pipe);
        self.channel_map.insert_new(Token(id), channel);
    }


    pub(crate) fn run(&self) {
        let selector = Arc::clone(&self.selector);
        let channel_map = Arc::clone(&self.channel_map);
        let channel_pipe_ctx_map = Arc::clone(&self.channel_handler_ctx_pipe_map);
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
                                // 将消息传到HEADER_HANDLER
                                let ctx_pipe = channel_pipe_ctx_map.get_mut(&e.token()).unwrap();
                                let mut ctx_head = ctx_pipe.header_handler_ctx();
                                let head_handler_clone = ctx_pipe.header_handler().clone();
                                let head_handler = head_handler_clone.lock().unwrap();
                                head_handler.channel_read(ctx_head, &bytebuf);
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
}

pub struct EventLoopGroup {
    group: Vec<EventLoop>,
}

impl EventLoopGroup {
    pub fn new(n: usize) -> EventLoopGroup {
        let mut _group = Vec::<EventLoop>::new();
        for _i in 0..n {
            _group.push(EventLoop::new());
        }
        EventLoopGroup {
            group: _group
        }
    }

    pub fn event_loop_group(&self) -> &Vec<EventLoop> {
        &self.group
    }
}

