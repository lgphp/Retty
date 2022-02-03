use std::any::Any;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

use bytebuf_rs::bytebuf::{ByteBuf, slice_util};

use crate::errors::RettyErrorKind;
use crate::handler::channel_handler_ctx::ChannelInboundHandlerCtx;
use crate::handler::handler::ChannelInboundHandler;

///
/// 第一个字段为长度字段的解码器
///
pub struct FirstIntegerLengthFieldDecoder {
    all_buf: Vec<u8>,
}


impl FirstIntegerLengthFieldDecoder {
    pub fn new() -> Self {
        FirstIntegerLengthFieldDecoder {
            all_buf: Vec::<u8>::new()
        }
    }
}


impl ChannelInboundHandler for FirstIntegerLengthFieldDecoder {
    fn id(&self) -> String {
        return "FirstIntegerLengthFieldDecoder".to_string();
    }

    fn channel_active(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {
        channel_handler_ctx.fire_channel_active();
    }

    fn channel_inactive(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx) {
        channel_handler_ctx.fire_channel_inactive();
    }

    fn channel_read(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, message: &mut dyn Any) {
        let buf_option = message.downcast_ref::<ByteBuf>();
        if buf_option.is_some() {
            let buf = buf_option.unwrap();
            let n = buf.len();
            if self.all_buf.len() != 0 {
                self.all_buf = slice_util::append::<u8>(&self.all_buf, &buf[..n]);
            } else {
                self.all_buf = buf[..n].to_owned();
            }
            let mut bytebuf = ByteBuf::new_from(&self.all_buf[..]);
            //处理粘包
            loop {
                if bytebuf.readable_bytes() < std::mem::size_of::<u32>() {
                    return;
                }
                bytebuf.mark_reader_index();
                let pkt_len = bytebuf.read_u32_be();
                bytebuf.reset_reader_index();
                if (bytebuf.readable_bytes() as u32) < pkt_len {
                    return;
                }
                channel_handler_ctx.fire_channel_read(&mut bytebuf);
                if bytebuf.get_reader_index() == bytebuf.get_writer_index() {
                    self.all_buf = Vec::<u8>::new();
                    return;
                }
            }
        } else {
            let err = RettyErrorKind::new(ErrorKind::Other, String::from("decoding error"));
            channel_handler_ctx.fire_channel_exception(err);
        }
    }

    fn channel_exception(&mut self, channel_handler_ctx: &mut ChannelInboundHandlerCtx, error: RettyErrorKind) {
        channel_handler_ctx.fire_channel_exception(error);
    }
}