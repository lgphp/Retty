use std::fmt::{Formatter, Pointer};
use std::sync::Arc;

use crate::handler::handler::{ChannelInboundHandler, ChannelOutboundHandler};

pub struct ChannelInboundHandlerPipe {
    pub handlers: Vec<Box<dyn ChannelInboundHandler + Send + Sync>>,
}

impl ChannelInboundHandlerPipe {
    pub fn new() -> ChannelInboundHandlerPipe {
        ChannelInboundHandlerPipe {
            handlers: Vec::new(),
        }
    }
    pub fn add_last(&mut self, handler: Box<dyn ChannelInboundHandler + Send + Sync>) {
        self.handlers.push(handler);
    }

    pub fn add_first(&mut self, handler: Box<dyn ChannelInboundHandler + Send + Sync>) {
        self.handlers.insert(0, handler);
    }
}


pub struct ChannelOutboundHandlerPipe {
    pub handlers: Vec<Box<dyn ChannelOutboundHandler + Send + Sync>>,
}

impl ChannelOutboundHandlerPipe {
    pub fn new() -> ChannelOutboundHandlerPipe {
        ChannelOutboundHandlerPipe {
            handlers: Vec::new(),
        }
    }
    pub fn add_last(&mut self, handler: Box<dyn ChannelOutboundHandler + Send + Sync>) {
        self.handlers.push(handler);
    }

    pub fn add_first(&mut self, handler: Box<dyn ChannelOutboundHandler + Send + Sync>) {
        self.handlers.insert(0, handler);
    }
}