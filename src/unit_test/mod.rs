use std::any::Any;
use std::sync::{Arc, Mutex};

use bytebuf_rs::bytebuf::ByteBuf;
use crossbeam::sync::WaitGroup;
use rayon_core::ThreadPool;
use uuid::Uuid;

use crate::core::bootstrap::Bootstrap;

#[test]
pub fn test_create_server() {}


struct A {
    s: Mutex<String>,
}


impl A {
    fn print_s(&self) {
        let guard = self.s.lock().unwrap();
        println!("s:{}", guard);
    }
}

#[test]
pub fn test_mutex() {
    let a = Mutex::new(A {
        s: Mutex::new(String::from("ssss"))
    });

    let a_obj = a.lock().unwrap();
    println!("1111");
    a_obj.print_s();
}


fn read(msg: &mut dyn Any) {
    let mut option = msg.downcast_mut::<ByteBuf>().unwrap();
    option.skip_index(2);
}

#[test]
pub fn test_byte_buf() {
    let mut buf = ByteBuf::new_with_capacity(10);

    buf.write_u32_be(1);

    read(&mut buf);
}

pub struct Ch {
    id: i8,
}

#[test]
pub fn test_ref() {
    let ch = Ch {
        id: 9i8
    };
    let ch = Arc::new(Mutex::new(ch));
    let ch1 = ch.clone();
    let ch2 = ch.clone();
    {
        let mut guard = ch1.lock().unwrap();
        guard.id = 8i8;
    }
    let mut guard_ch = ch2.lock().unwrap();
    println!("ch2: {:?}", guard_ch.id);
}

