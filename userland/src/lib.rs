#![no_std]
#![feature(start)]
#![feature(linkage)]

pub mod syscall;

#[panic_handler]
fn panic(_panic_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    #[linkage = "weak"]
    fn _main();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        _main();
    }
    loop {}
}
