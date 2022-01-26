### Introduction

Retty is a High performance I/O framework written by Rust inspired by Netty

### Feature

还没写完，刚实现了一小部分功能。 我会努力的。。。。。。

### Quick Start

```rust
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::thread;

use bytebuf_rs::bytebuf::ByteBuf;
use crossbeam_utils::sync::WaitGroup;
use rayon_core::ThreadPool;
use uuid::Uuid;

use retty::core::bootstrap::Bootstrap;
use retty::handler::channel_handler_ctx::ChannelHandlerCtx;
use retty::handler::handler::ChannelHandler;
use retty::handler::handler_pipe::ChannelHandlerPipe;

struct BizHandler {
    excutor: Arc<ThreadPool>,
}

impl BizHandler {
    fn new() -> Self {
        BizHandler {
            excutor: Arc::new(rayon_core::ThreadPoolBuilder::new().num_threads(1).build().unwrap())
        }
    }
}

impl ChannelHandler for BizHandler {
    fn id(&self) -> String {
        return "biz_handler".to_string();
    }

    fn channel_active(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>) {
        let mut ctx_clone = ctx.lock().unwrap();
        println!("业务处理 Handler --> : channel_active 新连接上线: {}", ctx_clone.channel().remote_addr().unwrap());
    }

    fn channel_read(&self, _ctx: Arc<Mutex<ChannelHandlerCtx>>, message: &dyn Any) {
        let msg = message.downcast_ref::<String>().unwrap();
        println!("业务处理 Handler  --> :收到消息:{}", msg);
        println!("reactor-excutor :{}", thread::current().name().unwrap());
        println!("========================================================");
    }
}


struct Decoder {
    excutor: Arc<ThreadPool>,
}

impl ChannelHandler for Decoder {
    fn id(&self) -> String {
        return "decoder_handler".to_string();
    }

    fn channel_active(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>) {
        let mut ctx_clone = ctx.lock().unwrap();
        println!("解码 Handler --> : channel_active 新连接上线: {}", ctx_clone.channel().remote_addr().unwrap());
        ctx_clone.fire_channel_active();
    }

    fn channel_read(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>, message: &dyn Any) {
        let msg = message.downcast_ref::<ByteBuf>().unwrap();
        println!("解码 Handler --> 收到Bytebuf:");
        msg.print_bytes();
        let mut ctx_clone = ctx.lock().unwrap();
        // 解码
        let obj = String::from_utf8_lossy(msg.available_bytes()).to_string();
        ctx_clone.fire_channel_read(&obj);
    }
}

impl Decoder {
    fn new() -> Self {
        Decoder {
            excutor: Arc::new(rayon_core::ThreadPoolBuilder::new().num_threads(1).build().unwrap())
        }
    }
}


fn main() {
    let mut bootstrap = Bootstrap::new_server_bootstrap();
    bootstrap.worker_group(8)
        .initialize_handler_pipeline(|| {
            let mut handler_pipe = ChannelHandlerPipe::new();
            let decoder_handler = Box::new(Decoder::new());
            let biz_handler = Box::new(BizHandler::new());
            handler_pipe.add_last(decoder_handler);
            handler_pipe.add_last(biz_handler);
            handler_pipe
        }).start();
    WaitGroup::new().clone().wait();
}
```
