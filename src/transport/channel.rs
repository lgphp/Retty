use std::any::Any;
use std::io::{Read, Result, Write};
use std::net::SocketAddr;
use std::sync::Arc;

use bytebuf_rs::bytebuf::ByteBuf;
use chashmap::CHashMap;
use mio::{Poll, PollOpt, Ready, Token};
use mio::net::TcpStream;
use rayon_core::ThreadPool;

use crate::core::eventloop::EventLoop;

#[derive(Clone)]
pub enum ChannelOptions {
    NUMBER(usize),
    BOOL(bool),
}


pub struct Channel {
    id: Token,
    stream: TcpStream,
    closed: bool,
    eventloop: Arc<EventLoop>,

}


impl Clone for Channel {
    fn clone(&self) -> Self {
        Channel {
            id: self.id.clone(),
            stream: self.stream.try_clone().unwrap(),
            closed: false,
            eventloop: self.eventloop.clone()
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

impl Channel {
    pub fn create(id: Token, eventloop: Arc<EventLoop>, stream: TcpStream,
    ) -> Channel {
        Channel {
            id,
            stream,
            closed: false,
            eventloop,
        }
    }


    pub fn remote_addr(&self) -> Result<SocketAddr> {
        self.stream.peer_addr()
    }


    // 从pipeline 最开始写
    // pub fn write(&mut self, message: &dyn Any) {
    //     println!("self.eventloop.channel_handler_ctx_pipe_map:size{}" , self.eventloop.channel_handler_ctx_pipe_map.len());
    //     let pipe = self.eventloop.channel_handler_ctx_pipe_map.get(&self.id).unwrap();
    //     pipe.head_channel_write(message);
    // }

    pub(crate) fn write_bytebuf(&mut self, buf: &ByteBuf) {
        self.stream.write(buf.available_bytes());
    }


    pub fn register(&self, poll: &Poll) {
        poll.register(
            &self.stream,
            self.id,
            Ready::readable(),
            PollOpt::edge(),
        );
    }
    pub fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        self.stream.read_to_end(buf)
    }


    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}