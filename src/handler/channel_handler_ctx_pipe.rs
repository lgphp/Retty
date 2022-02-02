use std::any::Any;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::errors::RettyErrorKind;
use crate::handler::channel_handler_ctx::{ChannelInboundHandlerCtx, ChannelOutboundHandlerCtx};
use crate::handler::handler::{ChannelInboundHandler, ChannelOutboundHandler};
use crate::handler::handler_pipe::{ChannelInboundHandlerPipe, ChannelOutboundHandlerPipe};

#[derive(Clone)]
pub(crate) struct ChannelInboundHandlerCtxPipe {
    pub(crate) channel_handler_ctx_pipe: Vec<Arc<Mutex<ChannelInboundHandlerCtx>>>,
    pub(crate) channel_handler_pipe: Vec<Arc<Mutex<Box<dyn ChannelInboundHandler + Send + Sync>>>>,
}

impl ChannelInboundHandlerCtxPipe {
    pub(crate) fn new() -> ChannelInboundHandlerCtxPipe {
        ChannelInboundHandlerCtxPipe {
            channel_handler_ctx_pipe: vec![],
            channel_handler_pipe: vec![],
        }
    }

    pub fn header_handler_ctx(&self) -> Arc<Mutex<ChannelInboundHandlerCtx>> {
        self.channel_handler_ctx_pipe.get(0).unwrap().clone()
    }
    pub fn header_handler(&self) -> Arc<Mutex<Box<dyn ChannelInboundHandler + Send + Sync>>> {
        self.channel_handler_pipe.get(0).unwrap().clone()
    }

    pub(crate) fn head_channel_read(&self, msg: &mut dyn Any) {
        let mut ctx_head = self.header_handler_ctx();
        let head_handler_clone = self.header_handler().clone();
        let mut head_handler = head_handler_clone.lock().unwrap();
        let mut ctx_head_ref = ctx_head.lock().unwrap();
        head_handler.channel_read(&mut ctx_head_ref, msg);
    }

    pub(crate) fn head_channel_active(&self) {
        let pipe = self.clone();
        let mut ctx_head = pipe.header_handler_ctx();
        let head_handler_clone = pipe.header_handler().clone();
        let mut head_handler = head_handler_clone.lock().unwrap();
        let mut ctx_head_ref = ctx_head.lock().unwrap();
        head_handler.channel_active(&mut *ctx_head_ref);
    }


    pub(crate) fn head_channel_exception(&self, error: RettyErrorKind) {
        let pipe = self.clone();
        let mut ctx_head = pipe.header_handler_ctx();
        let head_handler_clone = pipe.header_handler().clone();
        let mut head_handler = head_handler_clone.lock().unwrap();
        let mut ctx_head_ref = ctx_head.lock().unwrap();
        head_handler.channel_exception(&mut *ctx_head_ref, error);
    }


    pub(crate) fn head_channel_inactive(&self) {
        let pipe = self.clone();
        let mut ctx_head = pipe.header_handler_ctx();
        let head_handler_clone = pipe.header_handler().clone();
        let mut head_handler = head_handler_clone.lock().unwrap();
        let mut ctx_head_ref = ctx_head.lock().unwrap();
        head_handler.channel_inactive(&mut *ctx_head_ref);
    }


    pub(crate) fn add_last(&mut self, ctx: Arc<Mutex<ChannelInboundHandlerCtx>>, handler: Arc<Mutex<Box<dyn ChannelInboundHandler + Send + Sync>>>) {
        self.channel_handler_pipe.push(handler);
        self.channel_handler_ctx_pipe.push(ctx);
    }
}


#[derive(Clone)]
pub(crate) struct ChannelOutboundHandlerCtxPipe {
    pub(crate) channel_handler_ctx_pipe: Vec<Arc<Mutex<ChannelOutboundHandlerCtx>>>,
    pub(crate) channel_handler_pipe: Vec<Arc<Mutex<Box<dyn ChannelOutboundHandler + Send + Sync>>>>,
}

impl ChannelOutboundHandlerCtxPipe {
    pub(crate) fn new() -> ChannelOutboundHandlerCtxPipe {
        ChannelOutboundHandlerCtxPipe {
            channel_handler_ctx_pipe: vec![],
            channel_handler_pipe: vec![],
        }
    }

    pub fn header_handler_ctx(&self) -> Arc<Mutex<ChannelOutboundHandlerCtx>> {
        self.channel_handler_ctx_pipe.get(0).unwrap().clone()
    }
    pub fn header_handler(&self) -> Arc<Mutex<Box<dyn ChannelOutboundHandler + Send + Sync>>> {
        self.channel_handler_pipe.get(0).unwrap().clone()
    }


    pub(crate) fn head_channel_write(&self, msg: &mut dyn Any) {
        let pipe = self.clone();
        let ctx_head = pipe.header_handler_ctx();
        let head_handler_clone = pipe.header_handler().clone();
        let mut head_handler = head_handler_clone.lock().unwrap();
        let mut ctx_head_ref = ctx_head.lock().unwrap();
        head_handler.channel_write(&mut *ctx_head_ref, msg);
    }


    pub(crate) fn add_last(&mut self, ctx: Arc<Mutex<ChannelOutboundHandlerCtx>>, handler: Arc<Mutex<Box<dyn ChannelOutboundHandler + Send + Sync>>>) {
        self.channel_handler_pipe.push(handler);
        self.channel_handler_ctx_pipe.push(ctx);
    }
}