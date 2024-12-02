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
    NotComplete,
    Complete(u64),
    Error(Error),
}

try_from_enum! {
    pub enum Error : u64 {
        InvalidSyscall,
        InvalidArgument,
        NoSuchSocket,
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SyscallContext<'a> {
    pub syscall: Syscall<'a>,
    pub syscall_result: SyscallReturn,
}

impl<'a> SyscallContext<'a> {
    pub fn new(syscall: Syscall<'a>) -> Self {
        Self {
            syscall,
            syscall_result: SyscallReturn::NotComplete,
            tasks_to_wake: [0; 16],
            tasks_to_wake_count: None,
        }
    }
}

impl<'a> From<Syscall<'a>> for SyscallContext<'a> {
    fn from(syscall: Syscall<'a>) -> Self {
        Self::new(syscall)
    }
}
