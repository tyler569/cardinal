#![no_std]

#[repr(u32)]
pub enum Syscall {
    Print = 0,
}

#[repr(C)]
pub struct PrintArgs {
    pub data: *const [u8],
}
