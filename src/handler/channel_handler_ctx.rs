use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use rayon_core::ThreadPool;

use crate::core::eventloop::EventLoop;
use crate::handler::channel_handler_ctx_pipe::{ChannelInboundHandlerCtxPipe, ChannelOutboundHandlerCtxPipe};
use crate::handler::handler::{ChannelInboundHandler, ChannelOutboundHandler};
use crate::transport::channel::Channel;

/**
一个handlerctx 对应一个handler
 **/


pub struct ChannelInboundHandlerCtx {
    pub(crate) id: String,
    pub(crate) eventloop: Arc<EventLoop>,
    pub channel: Channel,
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
            channel,
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
            let next_handler = next_handler_arc.lock().unwrap();
            next_handler.channel_active(next_ctx_clone)
        }
    }


    pub fn fire_channel_read(&mut self, message: &dyn Any) {
        if self.next_ctx.is_some() {
            let next_ctx = self.next_ctx.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let next_handler = next_handler_arc.lock().unwrap();
            next_handler.channel_read(next_ctx_clone, message)
        }
    }




    pub(crate) fn channel_active(&mut self, ctx: Arc<Mutex<ChannelInboundHandlerCtx>>) {
        let current_ctx = ctx.lock().unwrap();
        let next_handler = current_ctx.handler.lock().unwrap();
        next_handler.channel_active(ctx.clone());
    }

    //
    // pub fn write(&self, message: &dyn Any) {
    //     let next_handler_arc = self.next_handler.as_ref().unwrap();
    //     let next_handler_clone = next_handler_arc.clone();
    //     let next_ctx_arc = self.next_ctx.as_ref().unwrap();
    //     let next_ctx_clone = next_ctx_arc.clone();
    //     let pipe = self.channel_handler_ctx_pipe.as_ref().unwrap();
    //     pipe.channel_write(next_ctx_clone, next_handler_clone, message);
    // }

    pub fn channel(&mut self) -> &mut Channel {
        let mut ch = &mut self.channel;
        ch
    }
}


///
/// 出站处理管道处理顺序与入站相反
///
pub struct ChannelOutboundHandlerCtx {
    pub(crate) id: String,
    pub(crate) eventloop: Arc<EventLoop>,
    pub(crate) channel: Channel,
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
            channel,
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
    pub fn fire_channel_write(&mut self, message: &dyn Any) {
        if self.next_ctx.is_some() {
            let next_ctx = self.next_ctx.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let next_handler = next_handler_arc.lock().unwrap();
            next_handler.channel_write(next_ctx_clone, message)
        }
    }

    pub fn channel(&mut self) -> &mut Channel {
        let mut ch = &mut self.channel;
        ch
    }

    pub fn id(&self) -> String {
        return self.id.clone();
    }
}