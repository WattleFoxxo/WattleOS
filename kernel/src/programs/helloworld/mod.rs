use crate::{println, serial_println, api::console};

pub async fn main() {
    console::clear();
    println!("Hello World");
    serial_println!("Hello World");
}