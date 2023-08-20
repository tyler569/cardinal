#![no_std]

#[repr(C)]
#[derive(Debug)]
pub enum Syscall<'a> {
    Println(&'a str),
    Exit(u32),
    Spawn(&'a str, usize),

    DgSocket,
    DgWrite(u64, &'a [u8]),
    DgRead(u64, *mut [u8]),
}
