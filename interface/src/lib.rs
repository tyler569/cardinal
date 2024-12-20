#![no_std]

#[macro_use] mod macros;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TaskId(pub u64);

#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum Syscall<'a> {
    Print(&'a str),
    Exit(u64),
    Spawn(&'a str, usize),

    Sleep(u64),

    DgSocket,
    DgWrite(u64, &'a [u8]),
    DgRead(u64, &'a mut [u8]),
    DgClose(u64),
    Yield,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SyscallReturn {
    Complete(u64),
    Error(Error),
    NotComplete,
}

try_from_enum! {
    pub enum Error : u64 {
        InvalidSyscall,
        InvalidArgument,
        NoSuchSocket,
    }
}
