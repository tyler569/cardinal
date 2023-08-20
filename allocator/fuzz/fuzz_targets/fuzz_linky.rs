#![no_main]
#![feature(allocator_api)]

extern crate std;

use cardinal3_allocator::linky;
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Copy, Clone, Debug)]
enum AllocatorMethod {
    Allocate { bytes: usize },
    Deallocate { index: usize },
}
use AllocatorMethod::*;

impl Arbitrary<'_> for AllocatorMethod {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let choice = u.int_in_range(0..=1)?;
        match choice {
            0 => Ok(Allocate { bytes: u.int_in_range(0..=0x1000)? }),
            1 => Ok(Deallocate { index: u.int_in_range(0..=0x1000)? }),
            _ => unreachable!(),
        }
    }
}

fuzz_target!(|ops: Vec<AllocatorMethod>| {
    let allocator = linky::new();
    unsafe { allocator.init() };
    let mut allocs: Vec<Option<Vec<u8, &linky::LockedAllocator>>> = Vec::new();
    for op in ops {
        match op {
            Allocate { bytes } => {
                allocs.push(Some(Vec::with_capacity_in(bytes, &allocator)));
            }
            Deallocate { index } => {
                if let Some(ptr) = allocs.get_mut(index) {
                    *ptr = None;
                }
            }
        }
    }
    for alloc in allocs.drain(..) {
        if let Some(alloc) = alloc {
            drop(alloc);
        }
    }
});
