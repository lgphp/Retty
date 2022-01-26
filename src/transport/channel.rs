use std::any::Any;
use std::io::{Read, Result, Write};
use std::net::SocketAddr;
use std::sync::Arc;

use mio::{Poll, PollOpt, Ready, Token};
use mio::net::TcpStream;
use rayon_core::ThreadPool;

#[derive(Clone)]
pub enum ChannelOptions {
    NUMBER(usize),
    BOOL(bool),
}


pub struct Channel {
    excutor: Arc<ThreadPool>,
    id: Token,
    stream: TcpStream,
    closed: bool,

}


impl Clone for Channel {
    fn clone(&self) -> Self {
        Channel {
            excutor: self.excutor.clone(),
            id: self.id.clone(),
            stream: self.stream.try_clone().unwrap(),
            closed: false,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

impl Channel {
    pub fn create(id: Token, excutor: Arc<ThreadPool>, stream: TcpStream,
    ) -> Channel {
        Channel {
            excutor,
            id,
            stream,
            closed: false,
        }
    }


    pub fn remote_addr(&self) -> Result<SocketAddr> {
        self.stream.peer_addr()
    }

    pub fn write(&mut self, message: &dyn Any) {
        let x = message.downcast_ref::<&[u8]>().unwrap();

        self.stream.write(x);
        println!("{:?}", x)
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

    // pub fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
    //     self.ctx.chan.read_to_end(buf)
    // }
}