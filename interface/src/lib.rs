#![no_std]

#[repr(C)]
#[derive(Debug)]
pub enum Syscall<'a> {
    Print(&'a str),
    Exit(u32),
}