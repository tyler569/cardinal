#![no_std]
#![no_main]

use cardinal3_interface::Syscall;
use cardinal3_userland::{executor, println, syscall};

#[no_mangle]
fn cardinal_main(_arg: usize) {
    println!("Hello World (from cardinal_main)");

    unsafe {
        executor::spawn(main());
        executor::run();
    }

    syscall::exit(0);
}

async fn main() {
    executor::syscall(Syscall::Print("Hello world from async 1!\n")).await;
    executor::syscall(Syscall::Print("Hello world from async 2!\n")).await;
}
