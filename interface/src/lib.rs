#![no_std]

#[macro_use]
extern crate num_derive;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TaskId(pub u64);

#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum Syscall<'a> {
    Println(&'a str),
    Exit(u64),
    Spawn(&'a str, usize),

    DgSocket,
    DgWrite(u64, &'a [u8]),
    DgRead(u64, &'a mut [u8]),
    DgClose(u64),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SyscallReturn {
    Complete(u64),
    Error(Error),
    NotComplete,
}

#[repr(u64)]
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Error {
    InvalidSyscall,
    InvalidArgument,
    NoSuchSocket,
}