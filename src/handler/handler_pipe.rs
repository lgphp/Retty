use std::fmt::{Formatter, Pointer};
use std::sync::Arc;

use crate::handler::handler::ChannelHandler;

pub struct ChannelHandlerPipe {
    pub handlers: Vec<Box<dyn ChannelHandler + Send + Sync>>,
}

impl ChannelHandlerPipe {
    pub fn new() -> ChannelHandlerPipe {
        ChannelHandlerPipe {
            handlers: Vec::new(),
        }
    }
    pub fn add_last(&mut self, handler: Box<dyn ChannelHandler + Send + Sync>) {
        self.handlers.push(handler);
    }

    pub fn add_first(&mut self, handler: Box<dyn ChannelHandler + Send + Sync>) {
        self.handlers.insert(0, handler);
    }
}

