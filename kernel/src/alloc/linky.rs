use crate::println;
use core::alloc::{AllocError, GlobalAlloc, Layout};
use core::fmt::{self, Debug, Formatter};
use core::mem::{size_of, transmute};
use core::ops::Deref;
use core::ptr::NonNull;
use spin::{Mutex, MutexGuard};

const PRINT: bool = false;

struct Allocator {
    head: Option<NonNull<Link>>,
    memory: [u8; 0x100000],
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Free,
    Allocated,
}

struct Link {
    next: Option<NonNull<Link>>,
    size: usize,
    state: State,
}

impl Link {
    fn memory(&self) -> *mut u8 {
        unsafe { (self as *const Link).offset(1) as *mut u8 }
    }
}

impl Debug for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Link {{ ")?;
        if let Some(ref next) = self.next {
            write!(f, "next: {:?}, ", unsafe { next.as_ref() })?;
        } else {
            write!(f, "next: None, ")?;
        }
        write!(f, "size: {}, ", self.size)?;
        write!(f, "state: {:?} ", self.state)?;
        write!(f, "}}")
    }
}

impl Allocator {
    pub const fn new() -> Self {
        Self {
            memory: [0; 0x100000],
            head: None,
        }
    }

    pub unsafe fn init(&mut self) {
        let head_ptr = transmute::<_, *mut Link>(self.memory.as_mut_ptr());
        *head_ptr = Link {
            next: None,
            size: self.memory.len() - size_of::<Link>(),
            state: State::Free,
        };
        self.head = Some(NonNull::new_unchecked(head_ptr));
    }

    fn split_region(&mut self, region: &mut Link, layout: Layout) {
        assert!(region.size >= layout.size());
        assert_eq!(region.state, State::Free);

        // round up the desired size to the nearest multiple of 16
        let size = layout.size();
        let size = (size + 15) & !15;

        // check if we have enough size to split
        if region.size < size + size_of::<Link>() + 16 {
            return;
        }

        // create a new region
        let new_region = unsafe {
            let new_region_ptr = region.memory().add(size) as *mut Link;
            *new_region_ptr = Link {
                next: region.next,
                size: region.size - size - size_of::<Link>(),
                state: State::Free,
            };
            NonNull::new_unchecked(new_region_ptr)
        };

        // update the current region
        region.next = Some(new_region);
        region.size = size;
        region.state = State::Free;
    }

    fn try_merge_regions(&mut self, regions: (&mut Link, &mut Link)) {
        let (first, second) = regions;
        if first.state != State::Free || second.state != State::Free {
            return;
        }

        first.next = second.next;
        first.size = first.size + second.size + size_of::<Link>();
    }

    fn merge_all_regions(&mut self) {
        let mut current = self.head;
        while let Some(mut region) = current {
            let region = unsafe { region.as_mut() };
            if let Some(mut next) = region.next {
                self.try_merge_regions((region, unsafe { next.as_mut() }));
            }
            current = region.next;
        }
    }

    fn allocate(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if PRINT {
            println!("allocating {:?}", layout);
        }

        let mut current = self.head;
        while let Some(mut region) = current {
            let region = unsafe { region.as_mut() };
            if region.size >= layout.size() && region.state == State::Free {
                // split the region
                self.split_region(region, layout);

                region.state = State::Allocated;

                // return the memory
                return Ok(unsafe {
                    NonNull::slice_from_raw_parts(
                        NonNull::new_unchecked(region.memory()),
                        layout.size(),
                    )
                });
            }
            current = region.next;
        }
        if PRINT {
            println!("failed to allocate {:?}, {:?}", layout, self.head);
        }
        Err(AllocError)
    }

    fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if PRINT {
            println!("deallocating {:?} {:?}", ptr, layout);
        }

        let region = unsafe { transmute::<_, *mut Link>(ptr.as_ptr()).offset(-1) };
        let region = unsafe { &mut *region };
        assert_eq!(region.state, State::Allocated);
        assert!(region.size >= layout.size());

        region.state = State::Free;

        if let Some(mut next) = region.next {
            self.try_merge_regions((region, unsafe { next.as_mut() }));
        }
        self.merge_all_regions();
    }
}

impl Debug for Allocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Allocator {{ ")?;
        write!(f, "memory: {:?}, ", self.memory)?;
        if let Some(ref head) = self.head {
            write!(f, "head: {:?}", unsafe { head.as_ref() })?;
        } else {
            write!(f, "head: None")?;
        }
        write!(f, " }}")
    }
}

pub struct LockedAllocator(Mutex<Allocator>);

impl LockedAllocator {
    fn lock(&self) -> MutexGuard<Allocator> {
        self.0.lock()
    }

    pub unsafe fn init(&self) {
        self.lock().init();
    }
}

impl Debug for LockedAllocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.lock().deref())
    }
}

unsafe impl Send for LockedAllocator {}
unsafe impl Sync for LockedAllocator {}

unsafe impl core::alloc::Allocator for LockedAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.lock().allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.lock().deallocate(ptr, layout)
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().allocate(layout).unwrap().as_ptr().as_mut_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

pub const fn new() -> LockedAllocator {
    LockedAllocator(Mutex::new(Allocator::new()))
}
