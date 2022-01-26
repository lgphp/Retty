use std::any::Any;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use rayon_core::ThreadPool;

use crate::handler::channel_handler_ctx_pipe::ChannelHandlerCtxPipe;
use crate::handler::handler::ChannelHandler;
use crate::transport::channel::Channel;

/**
一个handlerctx 对应一个handler
 **/
pub struct ChannelHandlerCtx {
    pub(crate) id: String,
    // id is handler's id
    // pub(crate) excutor: Arc<ThreadPool>,
    pub(crate) channel: Channel,
    pub(crate) handler: Arc<Mutex<Box<dyn ChannelHandler + Send + Sync>>>,
    pub(crate) next: Option<Arc<Mutex<ChannelHandlerCtx>>>,
    pub(crate) prev: Option<Arc<Mutex<ChannelHandlerCtx>>>,
    pub(crate) next_handler: Option<Arc<Mutex<Box<dyn ChannelHandler + Send + Sync>>>>,
}

impl ChannelHandlerCtx {
    pub fn new(id: String,
               channel: Channel,
               handler: Arc<Mutex<Box<dyn ChannelHandler + Send + Sync>>>,
    ) -> ChannelHandlerCtx {
        ChannelHandlerCtx {
            id,
            // excutor,
            // channel,
            channel,
            handler,
            next: None,
            prev: None,
            next_handler: None,
        }
    }

    pub(crate) fn id(&self) -> String {
        return self.id.clone();
    }


    pub fn fire_channel_active(&mut self) {
        if self.next.is_some() {
            let next_ctx = self.next.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let next_handler = next_handler_arc.lock().unwrap();
            next_handler.channel_active(next_ctx_clone)
        }
    }


    pub fn fire_channel_read(&mut self, message: &dyn Any) {
        if self.next.is_some() {
            let next_ctx = self.next.as_ref().unwrap();
            let next_ctx_clone = next_ctx.clone();
            let next_handler_arc = self.next_handler.as_ref().unwrap();
            let next_handler = next_handler_arc.lock().unwrap();
            next_handler.channel_read(next_ctx_clone, message)
        }
    }

    pub(crate) fn channel_active(&mut self, ctx: Arc<Mutex<ChannelHandlerCtx>>) {
        println!("channel_active.......{}", self.id());
        let msg = &"2222".as_bytes();
        let current_ctx = ctx.lock().unwrap();
        let next_handler = current_ctx.handler.lock().unwrap();
        next_handler.channel_active(ctx.clone());
    }

    pub fn channel(&mut self) -> &mut Channel {
        let mut ch = &mut self.channel;
        ch
    }

    // 从当前的handlerctx 往下写
    pub fn write(&mut self, message: &dyn Any) {

        // self.in_handler
    }
}

