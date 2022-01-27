use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::handler::channel_handler_ctx::ChannelInboundHandlerCtx;
use crate::handler::handler::ChannelInboundHandler;

pub struct LengthFieldBaseFrameDecoder {}

impl ChannelInboundHandler for LengthFieldBaseFrameDecoder {
    fn id(&self) -> String {
        todo!()
    }

    fn channel_active(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        todo!()
    }

    fn channel_inactive(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        todo!()
    }

    fn channel_read(&self, channel_handler_ctx: Arc<Mutex<ChannelInboundHandlerCtx>>, message: &dyn Any) {
        todo!()
    }
}