use core::cell::UnsafeCell;
use core::ffi::{c_char, CStr};

use super::{LIMINE_MAGIC1, LIMINE_MAGIC2};

#[repr(C)]
pub struct LimineModule {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: UnsafeCell<*mut LimineModuleResponse>,
}

impl LimineModule {
    pub const fn new() -> Self {
        Self {
            id: [
                LIMINE_MAGIC1,
                LIMINE_MAGIC2,
                0x3e7e279702be32af,
                0xca1c4f3bd1280cee,
            ],
            revision: 0,
            response: UnsafeCell::new(core::ptr::null_mut()),
        }
    }
}

unsafe impl Sync for LimineModule {}

#[repr(C)]
#[derive(Debug)]
pub struct LimineModuleResponse {
    pub revision: u64,
    pub module_count: u64,
    pub modules: *const *const LimineFile,
}

impl LimineModuleResponse {
    pub fn modules_slice(&self) -> &[*const LimineFile] {
        unsafe { core::slice::from_raw_parts(self.modules, self.module_count as usize) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct LimineFile {
    pub revision: u64,
    pub address: *mut u8,
    pub size: u64,
    pub path: *const c_char,
    pub cmdline: *const c_char,
    pub media_type: u32,
    pub unused: u32,
    pub tftp_ip: u32,
    pub tftp_port: u32,
    pub partition_index: u32,
    pub mbr_disk_id: u32,
    pub gpt_disk_uuid: u128,
    pub gpt_part_uuid: u128,
    pub part_uuid: u128,
}

impl LimineFile {
    pub fn path(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.path) }
    }

    pub fn cmdline(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.cmdline) }
    }

    pub fn data(&self) -> *const [u8] {
        unsafe { core::slice::from_raw_parts(self.address, self.size as usize) }
    }
}
