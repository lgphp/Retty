use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use rayon_core::ThreadPool;

use crate::core::eventloop::EventLoop;
use crate::errors::RettyErrorKind;
use crate::handler::channel_handler_ctx_pipe::{ChannelInboundHandlerCtxPipe, ChannelOutboundHandlerCtxPipe};
use crate::handler::handler::{ChannelInboundHandler, ChannelOutboundHandler};
use crate::transport::channel::{Channel, InboundChannelCtx, OutboundChannelCtx};

/**
一个handlerctx 对应一个handler
 **/


pub struct ChannelInboundHandlerCtx {
    pub(crate) id: String,
    pub(crate) eventloop: Arc<EventLoop>,
    pub(crate) channel_ctx: InboundChannelCtx,
    pub(crate) channel_handler_ctx_pipe: Option<ChannelInboundHandlerCtxPipe>,
    pub(crate) handler: Arc<Mutex<Box<dyn ChannelInboundHandler + Send + Sync>>>,

    pub(crate) head_ctx: Option<Arc<Mutex<ChannelInboundHandlerCtx>>>,
    pub(crate) next_ctx: Option<Arc<Mutex<ChannelInboundHandlerCtx>>>,

    pub(crate) head_handler: Option<Arc<Mutex<Box<dyn ChannelInboundHandler + Send + Sync>>>>,
    pub(crate) next_handler: Option<Arc<Mutex<Box<dyn ChannelInboundHandler + Send + Sync>>>>,
}

impl ChannelInboundHandlerCtx {
    pub fn new(id: String,
               eventloop: Arc<EventLoop>,
               channel: Channel,
               handler: Arc<Mutex<Box<dyn ChannelInboundHandler + Send + Sync>>>,
    ) -> ChannelInboundHandlerCtx {
        ChannelInboundHandlerCtx {
            id,
            eventloop,
            channel_ctx: InboundChannelCtx::new(channel),
            channel_handler_ctx_pipe: None,
            handler,
            next_ctx: None,
            next_handler: None,
            head_ctx: None,
            head_handler: None,
        }
    }

    pub fn id(&self) -> String {
        return self.id.clone();
    }


    pub fn fire_channel_active(&mut self) {
        if self.next_ctx.is_some() {
            let next_ctx = self.next_ctx.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let mut next_handler = next_handler_arc.lock().unwrap();
            let mut next_ctx_clone_ref = next_ctx_clone.lock().unwrap();
            next_handler.channel_active(&mut *next_ctx_clone_ref)
        }
    }

    pub fn fire_channel_inactive(&mut self) {
        if self.next_ctx.is_some() {
            let next_ctx = self.next_ctx.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let mut next_handler = next_handler_arc.lock().unwrap();
            let mut next_ctx_clone_ref = next_ctx_clone.lock().unwrap();
            next_handler.channel_inactive(&mut *next_ctx_clone_ref)
        }
    }

    pub fn fire_channel_read(&mut self, message: &mut dyn Any) {
        if self.next_ctx.is_some() {
            let next_ctx = self.next_ctx.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let mut next_handler = next_handler_arc.lock().unwrap();
            let mut next_ctx_clone_ref = next_ctx_clone.lock().unwrap();
            next_handler.channel_read(&mut *next_ctx_clone_ref, message)
        }
    }


    pub fn fire_channel_exception(&mut self, error: RettyErrorKind) {
        if self.next_ctx.is_some() {
            let next_ctx = self.next_ctx.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let mut next_handler = next_handler_arc.lock().unwrap();
            let mut next_ctx_clone_ref = next_ctx_clone.lock().unwrap();
            next_handler.channel_exception(&mut *next_ctx_clone_ref, error)
        }
    }


    pub(crate) fn channel_active(&mut self, ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        let current_ctx = ctx.lock().unwrap();
        let mut next_handler = current_ctx.handler.lock().unwrap();
        let ctx_ref_clone = ctx.clone();
        let mut ctx_ref_clone_ref = ctx_ref_clone.lock().unwrap();
        next_handler.channel_active(&mut *ctx_ref_clone_ref);
    }


    pub fn write_and_flush(&mut self, message: &mut dyn Any) {
        self.channel_ctx.write_and_flush(message);
    }

    pub fn channel(&mut self) -> &mut InboundChannelCtx {
        let mut ch_ctx = &mut self.channel_ctx;
        return ch_ctx;
    }

    pub fn close(&mut self) {
        self.channel_ctx.close()
    }

    pub fn event_loop(&mut self) -> Arc<EventLoop> {
        self.eventloop.clone()
    }
}


///
/// 出站处理管道处理顺序与入站相反
///
pub struct ChannelOutboundHandlerCtx {
    pub(crate) id: String,
    pub(crate) eventloop: Arc<EventLoop>,
    pub(crate) channel_ctx: OutboundChannelCtx,
    pub(crate) channel_handler_ctx_pipe: Option<ChannelOutboundHandlerCtxPipe>,
    pub(crate) handler: Arc<Mutex<Box<dyn ChannelOutboundHandler + Send + Sync>>>,

    ///
    /// 出站处理器 head 就是 tail
    ///
    pub(crate) head_ctx: Option<Arc<Mutex<ChannelOutboundHandlerCtx>>>,
    pub(crate) next_ctx: Option<Arc<Mutex<ChannelOutboundHandlerCtx>>>,
    pub(crate) head_handler: Option<Arc<Mutex<Box<dyn ChannelOutboundHandler + Send + Sync>>>>,
    pub(crate) next_handler: Option<Arc<Mutex<Box<dyn ChannelOutboundHandler + Send + Sync>>>>,

}

impl ChannelOutboundHandlerCtx {
    pub fn new(id: String,
               eventloop: Arc<EventLoop>,
               channel: Channel,
               handler: Arc<Mutex<Box<dyn ChannelOutboundHandler + Send + Sync>>>,
    ) -> ChannelOutboundHandlerCtx {
        ChannelOutboundHandlerCtx {
            id,
            eventloop,
            channel_ctx: OutboundChannelCtx::new(channel),
            channel_handler_ctx_pipe: None,
            handler,
            next_ctx: None,
            next_handler: None,
            head_ctx: None,
            head_handler: None,
        }
    }

    ///
    /// 从当前的ctx往下写
    ///
    pub fn fire_channel_write(&mut self, message: &mut dyn Any) {
        if self.next_ctx.is_some() {
            let next_ctx = self.next_ctx.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let mut next_handler = next_handler_arc.lock().unwrap();
            let mut next_ctx_clone_ref = next_ctx_clone.lock().unwrap();
            next_handler.channel_write(&mut *next_ctx_clone_ref, message)
        }
    }

    pub fn channel(&mut self) -> &mut OutboundChannelCtx {
        let mut ch_ctx = &mut self.channel_ctx;
        return ch_ctx;
    }

    pub fn id(&self) -> String {
        return self.id.clone();
    }

    pub fn event_loop(&mut self) -> Arc<EventLoop> {
        self.eventloop.clone()
    }
}