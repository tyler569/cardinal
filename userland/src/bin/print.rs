#![no_std]
#![no_main]

use cardinal3_userland::syscall;

#[no_mangle]
fn _main(arg: usize) {
    /*
    syscall::println("Hello, world!");
    syscall::exit(0);
     */

    /*
    if arg == 0 {
        syscall::println("Hello, world!");
        syscall::spawn("self", 1);
    } else {
        syscall::println("spawned");
    }
    syscall::exit(0);
     */

    /*
    if arg == 0 {
        syscall::println("Hello, world!");
        syscall::spawn("self", 1);
    } else if arg < 100 {
        syscall::spawn("self", arg + 1);
        syscall::spawn("self", 100);
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

    if arg == 0 {
        syscall::println("Hello World");
        let socket = syscall::socket();
        syscall::spawn("reader", socket as usize);
        syscall::write(socket, b"Hello World");
    } else {
        let data = &mut [0; 1024];
        syscall::async_read(arg as u64, data);

        for _ in 0..10 {
            syscall::println("waiting");
            if data[0] != 0 {
                syscall::println(unsafe { core::str::from_utf8_unchecked(&data[0..11]) });
                break;
            }
        }
    }

    syscall::exit(0);
}
