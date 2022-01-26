use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use chashmap::CHashMap;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::net::TcpListener;
use uuid::Uuid;

use crate::core::eventloop::EventLoopGroup;
use crate::handler::channel_handler_ctx::ChannelHandlerCtx;
use crate::handler::channel_handler_ctx_pipe::ChannelHandlerCtxPipe;
use crate::handler::handler::HeadHandler;
use crate::handler::handler_pipe::ChannelHandlerPipe;
use crate::transport::channel::{Channel, ChannelOptions};

pub struct Bootstrap {
    host: String,
    port: u16,
    boss_group: EventLoopGroup,
    worker_group: Option<Arc<EventLoopGroup>>,
    channel_handler_pipe_fn: Option<Arc<dyn Fn() -> ChannelHandlerPipe + Send + Sync + 'static>>,
    opts: HashMap<String, ChannelOptions>,
    stopped: Arc<AtomicBool>,

}


impl Bootstrap {
    pub fn new_server_bootstrap() -> Bootstrap {
        Bootstrap {
            host: "0.0.0.0".to_owned(),
            port: 1511,
            boss_group: EventLoopGroup::new(1),
            worker_group: None,
            channel_handler_pipe_fn: None,
            opts: HashMap::new(),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }


    pub fn initialize_handler_pipeline<F>(&mut self, pipe_fn: F) -> &mut Self
        where F: Fn() -> ChannelHandlerPipe + Send + Sync + 'static
    {
        self.channel_handler_pipe_fn = Some(Arc::new(Box::new(pipe_fn)));
        self
    }


    // 设置 worker_group
    pub fn worker_group(&mut self, n: usize) -> &mut Self {
        self.worker_group = Some(Arc::new(EventLoopGroup::new(n)));
        self
    }


    pub fn start(&self) {
        let boss_group = &self.boss_group;
        let boss_g = boss_group.event_loop_group().get(0).unwrap();

        let work_group = match &self.worker_group {
            None => panic!("work_group error"),
            Some(g) => Arc::clone(g),
        };


        let ip_addr = self.host.parse().unwrap();
        let sock_addr = Arc::new(SocketAddr::new(ip_addr, self.port));

        let opts = self.opts.clone();
        let stopped = Arc::clone(&self.stopped);

        let channel_handler_pipe_fn = &self.channel_handler_pipe_fn.as_ref().unwrap();
        let channel_handler_pipe_fn_clone = Arc::clone(channel_handler_pipe_fn);

        boss_g.excutor.spawn(move || {
            let mut events = Events::with_capacity(1024);
            let mut ch_id: usize = 1;
            let mut listener = match TcpListener::bind(&sock_addr) {
                Ok(s) => {
                    println!("[High performance I/O framework written by Rust inspired by Netty]");
                    println!("[Retty server is listening : {:?} : {:?}]", sock_addr.ip(), sock_addr.port());
                    s
                }
                Err(e) => {
                    println!("error : {:?}", e);
                    panic!("server is not started:{:?}", e)
                }
            };
            let mut sel = Poll::new().unwrap();
            // 将监听器绑定在selector上 , 打上Token(0)的标记，注册read事件, 也就是只监听Tcplistener的事件，后面是监听TcpStream的事件
            sel.register(&mut listener, Token(0), Ready::readable(), PollOpt::edge())
                .unwrap();
            // 循环event_loop,启动reactor线程
            work_group.event_loop_group().iter().for_each(|e| e.run());
            //当服务器没有停的时候
            while !stopped.load(Ordering::Relaxed) {
                // 取出selector中的事件集合
                match sel.poll(&mut events, None) {
                    Ok(_) => {}
                    Err(_) => {
                        continue;
                    }
                }
                // 循环事件，监听accept
                for _e in events.iter() {
                    let (mut sock, addr) = match listener.accept() {
                        Ok((s, a)) => (s, a),
                        Err(_) => {
                            continue;
                        }
                    };
                    // 创建ChannelHandlerPipe , 每一个连接创建自己的一套pipeline
                    let mut channel_handler_pipe: ChannelHandlerPipe = (channel_handler_pipe_fn_clone)();
                    //添加头handler
                    channel_handler_pipe.add_first(Box::new(HeadHandler::new()));
                    // 创建ChannelHandlerCtxPipe
                    let mut channel_handler_context_pipe = ChannelHandlerCtxPipe::new();
                    // 获得客户端连接，创建channel
                    let channel = Channel::create(Token(ch_id),
                                                  work_group.event_loop_group()[ch_id % work_group.event_loop_group().len()].excutor.clone(),
                                                  sock.try_clone().unwrap());

                    // 将这个ch注册到eventloop上的selector上， 一个eventloop里有多个channel
                    for _i in 0..channel_handler_pipe.handlers.len() {
                        let handler = channel_handler_pipe.handlers.remove(0);
                        let id = handler.id().clone();
                        let handler_arc = Arc::new(Mutex::new(handler));
                        let ctx = Arc::new(Mutex::new(ChannelHandlerCtx::new(id, channel.clone(), handler_arc.clone())));
                        channel_handler_context_pipe.add_last(ctx, handler_arc.clone());
                    }

                    let mut enumerate = channel_handler_context_pipe.channel_handler_ctx_pipe.iter().enumerate();
                    for _i in 0..channel_handler_context_pipe.channel_handler_ctx_pipe.len() {
                        let (_j, mut ctx) = enumerate.next().unwrap();
                        let prev_ctx = ctx.clone();
                        let mut curr = ctx.lock().unwrap();
                        let next_context = channel_handler_context_pipe.channel_handler_ctx_pipe.get(_j + 1);
                        match next_context {
                            Some((mut next_ctx)) => {
                                let next_ctx_clone = next_ctx.clone();
                                let mut next_ctx_guard = next_ctx.lock().unwrap();
                                curr.next = Some(next_ctx_clone);
                                curr.next_handler = Some(next_ctx_guard.handler.clone());
                                next_ctx_guard.prev = Some(prev_ctx);
                            }
                            None => {
                                curr.next = None
                            }
                        }
                    }
                    work_group.event_loop_group()[ch_id % work_group.event_loop_group().len()].attach(ch_id, channel.clone(), channel_handler_context_pipe);
                    ch_id = Bootstrap::incr_id(ch_id);
                }
            }
        });
    }

    #[inline]
    fn incr_id(cur_id: usize) -> usize {
        if cur_id >= usize::MAX {
            0
        } else {
            cur_id + 1
        }
    }
}

