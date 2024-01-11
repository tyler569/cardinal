#![no_std]
#![no_main]

use cardinal3_userland::{print, println, syscall};

#[no_mangle]
fn cardinal_main(_arg: usize) {
    println!("Hello World (from cardinal_main)");
    syscall::exit(0);
}
