use core::cell::UnsafeCell;
use core::ffi::{c_char, CStr};

use super::{LIMINE_MAGIC1, LIMINE_MAGIC2};

#[repr(C)]
pub struct LimineRsdp {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: UnsafeCell<*mut LimineRsdpResponse>,
}

impl LimineRsdp {
    pub const fn new() -> Self {
        Self {
            id: [
                LIMINE_MAGIC1,
                LIMINE_MAGIC2,
                0xc5e77b6b397e7b43,
                0x27637845accdcf3c,
            ],
            revision: 0,
            response: UnsafeCell::new(core::ptr::null_mut()),
        }
    }
}

unsafe impl Sync for LimineRsdp {}

#[repr(C)]
#[derive(Debug)]
pub struct LimineRsdpResponse {
    pub revision: u64,
    pub address: u64,
}
