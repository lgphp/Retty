use std::any::Any;
use std::collections::HashMap;
use std::io::{Read, Result, Write};
use std::net::Shutdown;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytebuf_rs::bytebuf::ByteBuf;
use chashmap::{CHashMap, ReadGuard};
use crossbeam::channel::{bounded, Receiver, select, Sender, tick};
use mio::{Poll, PollOpt, Ready, Token};
use mio::net::TcpStream;
use rayon_core::ThreadPool;

use crate::core::eventloop::EventLoop;
use crate::handler::channel_handler_ctx_pipe::ChannelOutboundHandlerCtxPipe;

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
    attribute: CHashMap<String, Arc<Mutex<Box<dyn Any + Send + Sync>>>>,
    inner_ch: (Sender<bool>, Receiver<bool>),
    last_read_time_ms: u64,
    read_idle_timeout_ms: u64,

}


impl Clone for Channel {
    fn clone(&self) -> Self {
        Channel {
            id: self.id.clone(),
            stream: self.stream.try_clone().unwrap(),
            closed: self.closed.clone(),
            eventloop: self.eventloop.clone(),
            attribute: self.attribute.clone(),
            inner_ch: self.inner_ch.clone(),
            last_read_time_ms: self.last_read_time_ms,
            read_idle_timeout_ms: self.read_idle_timeout_ms.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

impl Channel {
    pub fn create(id: Token, opts: HashMap<String, ChannelOptions>, eventloop: Arc<EventLoop>, stream: TcpStream,
    ) -> Channel {
        let tcp_stream = stream.try_clone().unwrap();
        let mut read_idle_timeout_ms = 50000u64;// 50 secs
        for (k, ref v) in opts.iter() {
            match k.as_ref() {
                "read_idle_timeout_ms" => {
                    match v {
                        ChannelOptions::NUMBER(read_idle_timeout) => {
                            read_idle_timeout_ms = (*read_idle_timeout as u64);
                        }
                        ChannelOptions::BOOL(_) => {}
                    }
                }
                "ttl" => {
                    match v {
                        ChannelOptions::NUMBER(ttl) => {
                            tcp_stream.set_ttl(*ttl as u32);
                        }
                        ChannelOptions::BOOL(_) => {}
                    }
                }
                "linger" => {
                    match v {
                        ChannelOptions::NUMBER(linger) => {
                            tcp_stream.set_linger(Some(Duration::from_millis(*linger as u64)));
                        }
                        ChannelOptions::BOOL(_) => {}
                    }
                }
                "nodelay" => {
                    match v {
                        ChannelOptions::NUMBER(_) => {}
                        ChannelOptions::BOOL(b) => {
                            tcp_stream.set_nodelay(*b);
                        }
                    }
                }
                "keep_alive" => {
                    match v {
                        ChannelOptions::NUMBER(keepalive) => {
                            tcp_stream.set_keepalive(Some(Duration::from_millis(*keepalive as u64)));
                        }
                        ChannelOptions::BOOL(_) => {}
                    }
                }
                "recv_buf_size" => {
                    match v {
                        ChannelOptions::NUMBER(bufsize) => {
                            tcp_stream.set_recv_buffer_size(*bufsize);
                        }
                        ChannelOptions::BOOL(_) => {}
                    }
                }
                "send_buf_size" => {
                    match v {
                        ChannelOptions::NUMBER(bufsize) => {
                            tcp_stream.set_send_buffer_size(*bufsize);
                        }
                        ChannelOptions::BOOL(_) => {}
                    }
                }
                _ => {}
            }
        }
        Channel {
            id,
            stream: tcp_stream,
            closed: false,
            eventloop,
            attribute: CHashMap::new(),
            inner_ch: bounded(1024),
            last_read_time_ms: 0,
            read_idle_timeout_ms: read_idle_timeout_ms.clone(),
        }
    }

    pub(crate) fn remote_addr(&self) -> Result<SocketAddr> {
        self.stream.peer_addr()
    }

    pub(crate) fn local_addr(&self) -> Result<SocketAddr> {
        self.stream.local_addr()
    }

    pub(crate) fn write_bytebuf(&mut self, buf: &ByteBuf) {
        self.stream.write(buf.available_bytes());
        self.stream.flush();
    }

    pub(crate) fn set_last_read_time(&mut self, ms: u64) {
        self.last_read_time_ms = ms;
    }

    pub(crate) fn last_read_time_ms(&self) -> u64 {
        self.last_read_time_ms
    }

    pub(crate) fn read_idle_timeout_ms(&self) -> u64 {
        self.read_idle_timeout_ms
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
        self.stream.shutdown(Shutdown::Both);
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

///
/// 暴露channel 用
///
pub struct InboundChannelCtx {
    pub(crate) channel: Arc<Mutex<Channel>>,
}

impl InboundChannelCtx {
    pub(crate) fn new(channel: Arc<Mutex<Channel>>) -> InboundChannelCtx {
        InboundChannelCtx {
            channel
        }
    }

    pub fn id(&self) -> String {
        let channel = self.channel.lock().unwrap();
        format!("{}", channel.id.0).clone()
    }
    pub fn set_attribute(&mut self, key: String, value: Box<dyn Any + Send + Sync>) {
        let channel = self.channel.lock().unwrap();
        channel.attribute.insert(key, Arc::new(Mutex::new(value)));
    }

    pub fn get_attribute(&self, key: String) -> Arc<Mutex<Box<dyn Any + Send + Sync>>> {
        let channel = self.channel.lock().unwrap();
        let v = channel.attribute.get(key.as_str()).unwrap();
        v.clone()
    }

    pub fn remote_addr(&self) -> Result<SocketAddr> {
        let channel = self.channel.lock().unwrap();
        channel.remote_addr()
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        let channel = self.channel.lock().unwrap();
        channel.local_addr()
    }


    pub fn is_active(&self) -> bool {
        let channel = self.channel.lock().unwrap();
        !channel.is_closed()
    }

    pub fn close(&mut self) {
        let mut channel = self.channel.lock().unwrap();
        channel.close()
    }

    pub(crate) fn set_last_read_time(&mut self, ms: u64) {
        let mut channel = self.channel.lock().unwrap();
        channel.last_read_time_ms = ms;
    }

    pub(crate) fn last_read_time_ms(&self) -> u64 {
        let channel = self.channel.lock().unwrap();
        channel.last_read_time_ms()
    }

    pub fn read_idle_timeout_ms(&self) -> u64 {
        let channel = self.channel.lock().unwrap();
        channel.read_idle_timeout_ms()
    }
}

pub struct OutboundChannelCtx {
    pub(crate) channel: Arc<Mutex<Channel>>,
}

impl OutboundChannelCtx {
    pub(crate) fn new(channel: Arc<Mutex<Channel>>) -> OutboundChannelCtx {
        OutboundChannelCtx {
            channel
        }
    }

    pub fn id(&self) -> String {
        let channel = self.channel.lock().unwrap();
        format!("{}", channel.id.0).clone()
    }

    pub(crate) fn write_bytebuf(&mut self, buf: &ByteBuf) {
        let mut channel = self.channel.lock().unwrap();
        channel.write_bytebuf(buf);
    }

    pub fn remote_addr(&self) -> Result<SocketAddr> {
        let channel = self.channel.lock().unwrap();
        channel.remote_addr()
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        let channel = self.channel.lock().unwrap();
        channel.local_addr()
    }

    pub fn is_active(&self) -> bool {
        let channel = self.channel.lock().unwrap();
        !channel.is_closed()
    }
}