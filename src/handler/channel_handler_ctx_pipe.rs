use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::handler::channel_handler_ctx::ChannelHandlerCtx;
use crate::handler::handler::ChannelHandler;

pub(crate) struct ChannelHandlerCtxPipe {
    pub(crate) channel_handler_ctx_pipe: Vec<Arc<Mutex<ChannelHandlerCtx>>>,
    pub(crate) channel_handler_pipe: Vec<Arc<Mutex<Box<dyn ChannelHandler + Send + Sync>>>>,
}

impl ChannelHandlerCtxPipe {
    pub(crate) fn new() -> ChannelHandlerCtxPipe {
        ChannelHandlerCtxPipe {
            channel_handler_ctx_pipe: vec![],
            channel_handler_pipe: vec![],
        }
    }

    pub(crate) fn header_handler_ctx(&self) -> Arc<Mutex<ChannelHandlerCtx>> {
        self.channel_handler_ctx_pipe.get(0).unwrap().clone()
    }
    pub(crate) fn header_handler(&self) -> Arc<Mutex<Box<dyn ChannelHandler + Send + Sync>>> {
        self.channel_handler_pipe.get(0).unwrap().clone()
    }

    pub(crate) fn add_last(&mut self, ctx: Arc<Mutex<ChannelHandlerCtx>>, handler: Arc<Mutex<Box<dyn ChannelHandler + Send + Sync>>>) {
        self.channel_handler_pipe.push(handler);
        self.channel_handler_ctx_pipe.push(ctx);
    }
}