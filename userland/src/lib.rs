#![no_std]
#![feature(start)]
#![feature(linkage)]
#![feature(waker_getters)]

extern crate alloc;

use alloc::string::ToString;
pub use cardinal3_allocator as allocator;

pub mod executor;
pub mod syscall;

#[global_allocator]
static ALLOCATOR: allocator::linky::LockedAllocator = allocator::linky::new();

pub fn static_heap_init() {
    unsafe {
        ALLOCATOR.init();
    }
}

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    syscall::println("user panic!");
    syscall::println(panic_info.to_string().as_str());
    syscall::exit(1);
    #[allow(unreachable_code)]
    loop {}
}

extern "Rust" {
    fn cardinal_main(arg: usize);
}

static mut N: usize = 0;

#[no_mangle]
pub extern "C" fn _start(arg: usize) {
    unsafe { N += 1 };
    syscall::println("userland starting...");
    assert_eq!(unsafe { N }, 1, "userland does not have clean bss pages!");
    static_heap_init();
    syscall::println("userland started!");
    unsafe {
        cardinal_main(arg);
    }
    syscall::println("main returned!");
    syscall::exit(0);
    #[allow(unreachable_code)]
    loop {}
}
