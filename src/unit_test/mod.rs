use std::any::Any;
use std::sync::{Arc, Mutex};

use crossbeam_utils::sync::WaitGroup;
use rayon_core::ThreadPool;
use uuid::Uuid;

use crate::core::bootstrap::Bootstrap;
use crate::handler::channel_handler_ctx::ChannelHandlerCtx;

#[test]
pub fn test_create_server() {
    let ctx = vec![0u8];
    let v = Arc::new(Mutex::new(Box: new(ctx)));
    let v1 = v.clone();
    let v2 = v1.clone();
}


fn read(v: Arc<Mutex<Box<Vec<u8>>>>) {}