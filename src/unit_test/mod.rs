use std::any::Any;
use std::sync::{Arc, Mutex};

use crossbeam_utils::sync::WaitGroup;
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


