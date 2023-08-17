#![no_main]
#![feature(allocator_api)]

extern crate std;

use cardinal3_allocator::linky;
use libfuzzer_sys::fuzz_target;

#[derive(Copy, Clone, Debug, Arbitrary)]
enum AllocatorMethod {
    Allocate { bytes: usize },
    Deallocate { index: usize },
}
use AllocatorMethod::*;

fuzz_target!(|ops: &[AllocatorMethod]| {
    let mut allocator = linky::new();
    let mut allocs: Vec<Option<Vec<u8, linky::LockedAllocator>>> = Vec::new();
    for op in ops {
        match op {
            &Allocate { bytes } => allocs.push(Some(Vec::with_capacity_in(bytes, allocator))),
            &Deallocate { index } => {
                if let Some(ptr) = allocs.get_mut(index) {
                    *ptr = None;
                }
            }
        }
    }
});
