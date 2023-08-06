use crate::allocator::linky;
use crate::allocator::linky::LockedAllocator;
use core::convert::AsMut;
use core::ptr::NonNull;
use spin::Lazy;

#[global_allocator]
static ALLOCATOR: LockedAllocator = linky::new();

pub fn static_heap_init() {
    unsafe {
        ALLOCATOR.init();
    }
}
