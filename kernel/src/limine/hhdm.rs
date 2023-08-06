use core::cell::UnsafeCell;
use core::ffi::{c_char, CStr};

use super::{LIMINE_MAGIC1, LIMINE_MAGIC2};

#[repr(C)]
pub struct LimineHhdm {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: UnsafeCell<*mut LimineHhdmResponse>,
}

impl LimineHhdm {
    pub const fn new() -> Self {
        Self {
            id: [
                LIMINE_MAGIC1,
                LIMINE_MAGIC2,
                0x48dcf1cb8ad2b852,
                0x63984e959a98244b,
            ],
            revision: 0,
            response: UnsafeCell::new(core::ptr::null_mut()),
        }
    }
}

unsafe impl Sync for LimineHhdm {}

#[repr(C)]
#[derive(Debug)]
pub struct LimineHhdmResponse {
    pub revision: u64,
    pub offset: u64,
}
