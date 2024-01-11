#![no_std]
#![feature(start)]
#![feature(linkage)]
#![feature(waker_getters)]

extern crate alloc;

pub use cardinal3_allocator as allocator;

pub mod executor;
pub mod format;
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
    print!("user panic: {}", panic_info);
    syscall::exit(1);
    #[allow(unreachable_code)]
    loop {
        print!("process returned to after panicking!");
    }
}

extern "Rust" {
    fn cardinal_main(arg: usize);
}

static mut N: usize = 0;

#[no_mangle]
pub extern "C" fn _start(arg: usize) {
    static_heap_init();
    println!("userland started..., N is {}", unsafe { N },);
    unsafe {
        cardinal_main(arg);
    }
    println!("main returned!");
    syscall::exit(0);
    #[allow(unreachable_code)]
    loop {}
}
