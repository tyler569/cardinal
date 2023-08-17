#![no_std]
#![feature(start)]
#![feature(linkage)]

pub mod syscall;

#[panic_handler]
fn panic(_panic_info: &core::panic::PanicInfo) -> ! {
    syscall::print("[user] panic!\n");
    syscall::exit(1);
    #[allow(unreachable_code)]
    loop {}
}

extern "Rust" {
    fn _main(arg: usize);
}

#[no_mangle]
pub extern "C" fn _start(arg: usize) {
    unsafe {
        _main(arg);
    }
    syscall::print("[user] main returned!\n");
    loop {}
}
