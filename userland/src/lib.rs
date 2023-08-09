#![no_std]
#![feature(start)]

mod syscall;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    syscall::print("Hello, world!\n");
    loop {}
}

#[panic_handler]
fn panic(_panic_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

