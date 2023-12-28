#![no_std]
#![feature(start)]
#![feature(linkage)]

extern crate alloc;

pub use cardinal3_allocator as allocator;

pub mod syscall;
pub mod executor;

#[global_allocator]
static ALLOCATOR: allocator::linky::LockedAllocator = allocator::linky::new();

pub fn static_heap_init() {
    unsafe {
        ALLOCATOR.init();
    }
}

#[panic_handler]
fn panic(_panic_info: &core::panic::PanicInfo) -> ! {
    syscall::println("user panic!\n");
    syscall::exit(1);
    #[allow(unreachable_code)]
    loop {}
}

extern "Rust" {
    fn _main(arg: usize);
}

#[no_mangle]
pub extern "C" fn _start(arg: usize) {
    static_heap_init();
    unsafe {
        _main(arg);
    }
    syscall::println("main returned!\n");
    syscall::exit(0);
    #[allow(unreachable_code)]
    loop {}
}
