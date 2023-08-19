#![no_std]
#![no_main]

use cardinal3_userland::syscall;

#[no_mangle]
fn _main(arg: usize) {
    /*
    syscall::println("Hello, world!");
    syscall::exit(0);
     */

    if arg == 0 {
        syscall::println("Hello, world!");
        syscall::spawn("self", 1);
    } else {
        syscall::println("spawned");
    }
    syscall::exit(0);

    /*
    if arg == 0 {
        syscall::println("Hello, world!");
        syscall::spawn("self", 1);
    } else if arg == 1 {
        syscall::spawn("self", 99);
    } else if arg > 4 && arg < 100 {
        syscall::spawn("self", arg - 1);
    } else {
        syscall::println("spawned");
    }
    syscall::exit(0);
     */

    /*
    syscall::spawn("self", arg * 2);
    syscall::spawn("self", arg * 2 + 1);
    syscall::exit(0);
     */
}
