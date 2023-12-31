use core::cell::UnsafeCell;
use core::fmt::Formatter;

use super::{LIMINE_MAGIC1, LIMINE_MAGIC2};

#[repr(C)]
pub struct LimineSmp {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: UnsafeCell<*mut LimineSmpResponse>,
    pub flags: u32,
}

impl LimineSmp {
    pub const ENABLE_X2APIC: u32 = 0x01;

    pub const fn new(flags: u32) -> Self {
        Self {
            id: [
                LIMINE_MAGIC1,
                LIMINE_MAGIC2,
                0x95a67b819a1b857e,
                0xa0b61b723b6a73e0,
            ],
            revision: 0,
            response: UnsafeCell::new(core::ptr::null_mut()),
            flags,
        }
    }
}

unsafe impl Sync for LimineSmp {}

#[repr(C)]
#[derive(Debug)]
pub struct LimineSmpResponse {
    pub revision: u64,
    pub flags: u32,
    pub bsp_lapic_id: u32,
    pub cpu_count: u64,
    pub cpus: *const *mut LimineCpuInfo,
}

impl LimineSmpResponse {
    pub const ENABLED_X2APIC: u32 = 0x01;

    pub fn cpus_slice(&self) -> &[*mut LimineCpuInfo] {
        // SAFETY: the bootloader populates `self.cpus` with `self.cpu_count` valid pointers
        unsafe { core::slice::from_raw_parts(self.cpus, self.cpu_count as usize) }
    }
}

#[repr(C)]
pub struct LimineCpuInfo {
    pub processor_id: u32,
    pub lapic_id: u32,
    pub reserved: u64,
    pub goto_address: UnsafeCell<unsafe extern "C" fn(*const LimineCpuInfo) -> !>,
    pub extra_argument: u64,
}

impl core::fmt::Debug for LimineCpuInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LimineCpuInfo")
            .field("processor_id", &self.processor_id)
            .field("lapic_id", &self.lapic_id)
            .field("goto_address", &self.goto_address)
            .field("extra_argument", &self.extra_argument)
            .finish()
    }
}
