#![no_std]

#[repr(C)]
#[derive(Debug)]
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
    DgRead(u64, *mut [u8]),

    // DgWriteAsync(u64, &'a [u8], TaskId),
    ReadAsync(u64, &'a [u8], TaskId),
    // Wait(&'a [TaskId])
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    EAGAIN = 1,
    EINVAL = 2,
}

impl Error {
    pub fn return_value(self) -> usize {
        !(self as usize)
    }
}
