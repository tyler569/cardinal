#![no_std]
#![no_main]

use cardinal3_userland::syscall;

#[no_mangle]
fn _main(arg: usize) {
    if arg == 0 {
        syscall::print("[user] Hello, world!\n");
        syscall::spawn("", 1);
    } else {
        syscall::print("[user] spawned\n");
    }
    syscall::exit(0);
}
