#![no_std]
#![no_main]

use cardinal3_userland::syscall;

#[no_mangle]
fn _main() {
    syscall::print("Hello, world!\n");
    syscall::exit();
}