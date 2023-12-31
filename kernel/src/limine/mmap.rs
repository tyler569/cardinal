use core::cell::UnsafeCell;

use super::{LIMINE_MAGIC1, LIMINE_MAGIC2};

#[repr(C)]
pub struct LimineMmap {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: UnsafeCell<*mut LimineMmapResponse>,
}

impl LimineMmap {
    pub const fn new() -> Self {
        Self {
            id: [
                LIMINE_MAGIC1,
                LIMINE_MAGIC2,
                0x67cf3d9d378a806f,
                0xe304acdfc50c3c62,
            ],
            revision: 0,
            response: UnsafeCell::new(core::ptr::null_mut()),
        }
    }
}

unsafe impl Sync for LimineMmap {}

#[repr(C)]
#[derive(Debug)]
pub struct LimineMmapResponse {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *const *const LimineMmapEntry,
}

impl LimineMmapResponse {
    pub fn entries_slice(&self) -> &[*const LimineMmapEntry] {
        // SAFETY: the bootloader populates `self.entries` with `self.entry_count` valid pointers
        unsafe { core::slice::from_raw_parts(self.entries, self.entry_count as usize) }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LimineMmapEntryType {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    BadMemory = 4,
    BootloaderReclaimable = 5,
    KernelAndModules = 6,
    Framebuffer = 7,
}

#[repr(C)]
#[derive(Debug)]
pub struct LimineMmapEntry {
    pub base: u64,
    pub len: u64,
    pub typ: LimineMmapEntryType,
}
