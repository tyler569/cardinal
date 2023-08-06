use core::cell::UnsafeCell;
use core::ffi::{c_char, CStr};
use core::fmt::Formatter;

use super::{LIMINE_MAGIC1, LIMINE_MAGIC2};

#[repr(C)]
pub struct LimineBootInfo {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: UnsafeCell<*mut LimineBootInfoResponse>,
}

impl LimineBootInfo {
    pub const fn new() -> Self {
        Self {
            id: [
                LIMINE_MAGIC1,
                LIMINE_MAGIC2,
                0xf55038d8e2a1202f,
                0x279426fcf5f59740,
            ],
            revision: 0,
            response: UnsafeCell::new(core::ptr::null_mut()),
        }
    }
}

unsafe impl Sync for LimineBootInfo {}

#[repr(C)]
pub struct LimineBootInfoResponse {
    pub revision: u64,
    pub name: *const c_char,
    pub version: *const c_char,
}

impl LimineBootInfoResponse {
    pub fn name(&self) -> &CStr {
        // SAFETY: the bootloader populates `self.name` with a valid pointer to a valid string
        unsafe { CStr::from_ptr(self.name) }
    }

    pub fn version(&self) -> &CStr {
        // SAFETY: the bootloader populates `self.version` with a valid pointer to a valid string
        unsafe { CStr::from_ptr(self.version) }
    }
}

impl core::fmt::Debug for LimineBootInfoResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LimineBootInfoResponse")
            .field("revision", &self.revision)
            .field("name", &self.name())
            .field("version", &self.version())
            .finish()
    }
}
