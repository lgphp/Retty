use std::any::Any;
use std::sync::{Arc, Mutex};

use bytebuf_rs::bytebuf::ByteBuf;

use crate::handler::channel_handler_ctx::{ChannelInboundHandlerCtx, ChannelOutboundHandlerCtx};

pub trait ChannelInboundHandler {
    fn id(&self) -> String;
    fn channel_active(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>);
    fn channel_read(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>, message: &dyn Any);
}


pub trait ChannelOutboundHandler {
    fn id(&self) -> String;
    fn channel_write(&self, channel_handler_ctx: Arc<Mutex<ChannelOutboundHandlerCtx>>, message: &dyn Any);
}


pub(crate) struct HeadHandler {}

impl ChannelInboundHandler for HeadHandler {
    fn id(&self) -> String {
        return String::from("HEAD");
    }

    fn channel_active(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        let mut ctx = channel_handler_ctx.lock().unwrap();
        ctx.fire_channel_active();
    }
    fn channel_read(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>, message: &dyn Any) {
        let mut ctx = channel_handler_ctx.lock().unwrap();
        ctx.fire_channel_read(message);
    }
}

impl HeadHandler {
    pub(crate) fn new() -> HeadHandler {
        HeadHandler {}
    }
}


pub(crate) struct TailHandler {}


impl ChannelOutboundHandler for TailHandler {
    fn id(&self) -> String {
        return String::from("TAIL");
    }

    fn channel_write(&self, channel_handler_ctx: Arc<Mutex<ChannelOutboundHandlerCtx>>, message: &dyn Any) {
        // let mut ctx = channel_handler_ctx.lock().unwrap();
        // let bytes = message.downcast_ref::<ByteBuf>();
        // match bytes {
        //     Some(buf) =>{
        //         ctx.channel().write_bytebuf(buf);
        //     },
        //     None=>{
        //         println!("TailHandler message is not bytebuf");
        //     }
        // }
    }
}

impl TailHandler {
    pub(crate) fn new() -> TailHandler {
        TailHandler {}
    }
}