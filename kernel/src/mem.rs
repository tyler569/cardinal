use crate::allocator::linky;
use crate::allocator::linky::LockedAllocator;

#[global_allocator]
static ALLOCATOR: LockedAllocator = linky::new();

pub fn static_heap_init() {
    unsafe {
        ALLOCATOR.init();
    }
}
