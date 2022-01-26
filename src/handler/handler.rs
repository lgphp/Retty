use std::any::Any;
use std::sync::{Arc, Mutex};

use bytebuf_rs::bytebuf::ByteBuf;

use crate::handler::channel_handler_ctx::ChannelHandlerCtx;

pub trait ChannelHandler {
    fn id(&self) -> String;
    fn channel_active(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>);
    fn channel_read(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>, message: &dyn Any);
}


pub(crate) struct HeadHandler {}


impl ChannelHandler for HeadHandler {
    fn id(&self) -> String {
        return String::from("HEAD");
    }

    fn channel_active(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>) {
        let mut ctx_clone = ctx.lock().unwrap();
        ctx_clone.fire_channel_active();
    }
    fn channel_read(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>, message: &dyn Any) {
        let mut ctx_clone = ctx.lock().unwrap();
        ctx_clone.fire_channel_read(message);
    }
}

impl HeadHandler {
    pub(crate) fn new() -> HeadHandler {
        HeadHandler {}
    }
}


pub(crate) struct TailHandler {}


impl ChannelHandler for TailHandler {
    fn id(&self) -> String {
        todo!()
    }

    fn channel_active(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>) {
        todo!()
    }

    fn channel_read(&self, ctx: Arc<Mutex<ChannelHandlerCtx>>, message: &dyn Any) {
        todo!()
    }
}
