use std::any::Any;
use std::sync::{Arc, Mutex};
use std::thread;

use bytebuf_rs::bytebuf::ByteBuf;
use crossbeam_utils::sync::WaitGroup;
use rayon_core::ThreadPool;
use uuid::Uuid;

use retty::core::bootstrap::Bootstrap;
use retty::handler::channel_handler_ctx::{ChannelInboundHandlerCtx, ChannelOutboundHandlerCtx};
use retty::handler::handler::{ChannelInboundHandler, ChannelOutboundHandler};
use retty::handler::handler_pipe::{ChannelInboundHandlerPipe, ChannelOutboundHandlerPipe};

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

impl ChannelInboundHandler for BizHandler {
    fn id(&self) -> String {
        return "biz_handler".to_string();
    }

    fn channel_active(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        let mut ctx = channel_handler_ctx.lock().unwrap();
        let addr = ctx.channel().remote_addr().unwrap();
        println!("业务处理 Handler --> : channel_active 新连接上线: {}", addr);
        ctx.write_and_flush(&format!("::: 欢迎你:==>{}", addr))
    }

    fn channel_inactive(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        let mut ctx = channel_handler_ctx.lock().unwrap();
        println!("远端断开连接： Inactive: remote_addr: {}", ctx.channel().remote_addr().unwrap())
    }

    fn channel_read(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>, message: &dyn Any) {
        let msg = message.downcast_ref::<String>().unwrap();
        println!("业务处理 Handler  --> :收到消息:{}", msg);
        println!("reactor-excutor :{}", thread::current().name().unwrap());
        let mut ctx = channel_handler_ctx.lock().unwrap();
        ctx.write_and_flush(&format!("::: I Love You !!!! :==>{}", msg));
        println!("========================================================");
    }
}


struct Decoder {
    excutor: Arc<ThreadPool>,
}

impl ChannelInboundHandler for Decoder {
    fn id(&self) -> String {
        return "encoder_handler".to_string();
    }

    fn channel_active(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        let mut ctx = channel_handler_ctx.lock().unwrap();
        println!("解码 Handler --> : channel_active 新连接上线: {}", ctx.channel().remote_addr().unwrap());
        ctx.fire_channel_active();
    }

    fn channel_inactive(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        let mut ctx = channel_handler_ctx.lock().unwrap();
        ctx.fire_channel_inactive()
    }

    fn channel_read(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>, message: &dyn Any) {
        let msg = message.downcast_ref::<ByteBuf>().unwrap();
        println!("解码 Handler --> 收到Bytebuf:");
        msg.print_bytes();
        let mut ctx = channel_handler_ctx.lock().unwrap();
        // 解码
        let obj = String::from_utf8_lossy(msg.available_bytes()).to_string();
        ctx.fire_channel_read(&obj);
    }
}

impl Decoder {
    fn new() -> Self {
        Decoder {
            excutor: Arc::new(rayon_core::ThreadPoolBuilder::new().num_threads(1).build().unwrap())
        }
    }
}


struct Encoder {
    excutor: Arc<ThreadPool>,
}

impl ChannelOutboundHandler for Encoder {
    fn id(&self) -> String {
        return "encoder_handler".to_string();
    }


    fn channel_write(&self, channel_handler_ctx: Arc<Mutex<ChannelOutboundHandlerCtx>>, message: &dyn Any) {
        let mut ctx = channel_handler_ctx.lock().unwrap();

        let msg = message.downcast_ref::<String>().unwrap();
        println!("回执消息，编码器 ：====>Encoder Handler:{}", msg);
        let buf = ByteBuf::new_from(msg.as_bytes());
        ctx.fire_channel_write(&buf);
    }
}

impl Encoder {
    fn new() -> Self {
        Encoder {
            excutor: Arc::new(rayon_core::ThreadPoolBuilder::new().num_threads(1).build().unwrap())
        }
    }
}


fn main() {
    let mut bootstrap = Bootstrap::new_server_bootstrap();
    bootstrap.worker_group(8)
        .bind("0.0.0.0", 1512)
        .opt_ttl_ms(1000)
        .initialize_inbound_handler_pipeline(|| {
            let mut handler_pipe = ChannelInboundHandlerPipe::new();
            let decoder_handler = Box::new(Decoder::new());
            let biz_handler = Box::new(BizHandler::new());
            handler_pipe.add_last(decoder_handler);
            handler_pipe.add_last(biz_handler);
            handler_pipe
        }).initialize_outbound_handler_pipeline(|| {
        let mut handler_pipe = ChannelOutboundHandlerPipe::new();
        let encoder_handler = Box::new(Encoder::new());
        handler_pipe.add_last(encoder_handler);
        handler_pipe
    })

        .start();
    WaitGroup::new().clone().wait();
}