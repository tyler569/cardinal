// Much of this comes in over FFI and/or exposes features of the limine API that I may
// or may not be using yet.
#![allow(dead_code)]

pub mod boot_info;
pub mod hhdm;
pub mod mmap;
pub mod module;
pub mod rsdp;
pub mod smp;

const LIMINE_MAGIC1: u64 = 0xc7b1dd30df4c8b88;
const LIMINE_MAGIC2: u64 = 0x0a82e883a194f07b;

pub static BOOT_INFO: boot_info::LimineBootInfo = boot_info::LimineBootInfo::new();
pub static HHDM: hhdm::LimineHhdm = hhdm::LimineHhdm::new();
pub static MMAP: mmap::LimineMmap = mmap::LimineMmap::new();
pub static RSDP: rsdp::LimineRsdp = rsdp::LimineRsdp::new();
pub static SMP: smp::LimineSmp = smp::LimineSmp::new(0);
pub static MODULE: module::LimineModule = module::LimineModule::new();
