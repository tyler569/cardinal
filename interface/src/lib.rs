#![no_std]

#[repr(C)]
#[derive(Debug)]
pub enum Syscall<'a> {
    Println(&'a str),
    Exit(u32),
    Spawn(&'a str, usize),
}
