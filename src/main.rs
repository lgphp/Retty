use std::any::Any;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::thread;

use bytebuf_rs::bytebuf::ByteBuf;
use crossbeam::sync::WaitGroup;
use rayon_core::ThreadPool;
use uuid::Uuid;

use retty::core::bootstrap::Bootstrap;
use retty::core::eventloop::EventLoopGroup;
use retty::errors::RettyErrorKind;
use retty::handler::channel_handler_ctx::{ChannelInboundHandlerCtx, ChannelOutboundHandlerCtx};
use retty::handler::codec::first_integer_length_field_decoder::FirstIntegerLengthFieldDecoder;
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

    fn channel_active(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {
        let addr = channel_handler_ctx.channel().remote_addr().unwrap();
        println!("业务处理 Handler --> : channel_active 新连接上线: {}", addr);
        channel_handler_ctx.write_and_flush(&mut format!("::: 欢迎你:==>{}", addr));
        let attr = channel_handler_ctx.channel().get_attribute("User".to_string());
        let attr = attr.lock().unwrap();
        let attr = attr.downcast_ref::<String>().unwrap();
        println!("========================================================:att:::: {}", attr);
    }

    fn channel_inactive(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {
        println!("is_active:{}", channel_handler_ctx.channel().is_active());
        println!("远端断开连接： Inactive: channel_id : {}", channel_handler_ctx.channel().id())
    }

    fn channel_read(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, message: &mut dyn Any) {
        let msg = message.downcast_ref::<String>().unwrap();
        println!("业务处理 Handler  --> :收到消息:{}", msg);
        println!("reactor-excutor :{}", thread::current().name().unwrap());
        channel_handler_ctx.write_and_flush(&mut format!("::: I Love You !!!! :==>{}", msg));
        let attr = channel_handler_ctx.channel().get_attribute("User".to_string());
        let attr = attr.lock().unwrap();
        let attr = attr.downcast_ref::<String>().unwrap();
        println!("========================================================:att:::: {}", attr);
    }

    fn channel_exception(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, error: RettyErrorKind) {
        channel_handler_ctx.fire_channel_exception(error);
    }
}


struct Decoder {
    excutor: Arc<ThreadPool>,
}

impl ChannelInboundHandler for Decoder {
    fn id(&self) -> String {
        return "decoder_handler".to_string();
    }

    fn channel_active(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {
        // set attribute
        channel_handler_ctx.channel().set_attribute("User".to_string(), Box::new("lgphp".to_string()));
        println!("解码 Handler --> : channel_active 新连接上线: {}", channel_handler_ctx.channel().remote_addr().unwrap());
        channel_handler_ctx.fire_channel_active();
    }

    fn channel_inactive(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {
        channel_handler_ctx.fire_channel_inactive()
    }

    fn channel_read(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, message: &mut dyn Any) {
        let mut buf = message.downcast_mut::<ByteBuf>().unwrap();
        println!("解码 Handler --> 收到Bytebuf:");
        // 解码
        let _pkt_len = buf.read_u32_be();
        let _ver = buf.read_u32_be();
        let mut msg = buf.read_string_with_u8_be_len();
        channel_handler_ctx.fire_channel_read(&mut msg);
    }

    fn channel_exception(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, error: RettyErrorKind) {
        channel_handler_ctx.fire_channel_exception(error);
    }
}

impl Decoder {
    fn new() -> Self {
        Decoder {
            excutor: Arc::new(rayon_core::ThreadPoolBuilder::new().num_threads(1).build().unwrap())
        }
    }
}

///
/// 入站异常handler 通常在最后一个
///
struct InboundExceptionHandler {}

impl ChannelInboundHandler for InboundExceptionHandler {
    fn id(&self) -> String {
        String::from("InboundExceptionHandler")
    }

    fn channel_active(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {}

    fn channel_inactive(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {}

    fn channel_read(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, message: &mut dyn Any) {}

    fn channel_exception(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, error: RettyErrorKind) {
        let mut ch = channel_handler_ctx.channel();

        // 处理 ReadIdleTimeout

        if error.kind == ErrorKind::TimedOut {
            println!("channel_id:{} 在 {}", ch.id(), format!("{} ms 没有读到数据！ , error_message:{}", ch.read_idle_timeout_ms(), error.message));
            ch.close()
        }
    }
}

impl InboundExceptionHandler {
    fn new() -> Self {
        InboundExceptionHandler {}
    }
}


struct Encoder {
    excutor: Arc<ThreadPool>,
}

impl ChannelOutboundHandler for Encoder {
    fn id(&self) -> String {
        return "encoder_handler".to_string();
    }


    fn channel_write(&mut self, channel_handler_ctx: &mut ChannelOutboundHandlerCtx, message: &mut dyn Any) {
        let msg = message.downcast_ref::<String>().unwrap();
        println!("回执消息，编码器 ：====>Encoder Handler:{}", msg);
        let mut buf = ByteBuf::new_with_capacity(0);
        let re = format!("回执消息，编码器 ：====>Encoder Handler:{}", msg);
        buf.write_u32_be((1 + re.as_bytes().len()) as u32);
        buf.write_string_with_u8_be_len(re);
        channel_handler_ctx.fire_channel_write(&mut buf);
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
        .opt_keep_alive_ms(30000)
        .opt_nodelay(false)
        .opt_send_buf_size(65535)
        .opt_recv_buf_size(65535)
        .opt_read_idle_timeout_ms(3000)
        .initialize_inbound_handler_pipeline(|| {
            let mut handler_pipe = ChannelInboundHandlerPipe::new();
            let decoder_handler = Box::new(Decoder::new());
            let biz_handler = Box::new(BizHandler::new());
            let excetion_handler = Box::new(InboundExceptionHandler::new());
            handler_pipe.add_last(Box::new(FirstIntegerLengthFieldDecoder::new()));
            handler_pipe.add_last(decoder_handler);
            handler_pipe.add_last(biz_handler);
            handler_pipe.add_last(excetion_handler);
            handler_pipe
        })
        .initialize_outbound_handler_pipeline(|| {
            let mut handler_pipe = ChannelOutboundHandlerPipe::new();
            let encoder_handler = Box::new(Encoder::new());
            handler_pipe.add_last(encoder_handler);
            handler_pipe
        }).start();

    // use  default_event_loop
    let mut new_default_event_loop_group = EventLoopGroup::new_default_event_loop_group(9);
    new_default_event_loop_group.execute(|| {
        println!(" default_event_loop  execute Task ..... is here")
    });
    WaitGroup::new().clone().wait();
}