use cardinal3_interface::{Syscall, TaskId};
use core::arch::asm;

fn syscall(args: &Syscall) -> usize {
    let result: usize;
    unsafe {
        asm!(
            "int 0x80",
            inout("rax") args as *const _ as usize => result,
            options(nostack)
        );
    }
    result
}

pub fn println(string: &str) -> usize {
    syscall(&Syscall::Println(string))
}

pub fn exit(code: u64) -> ! {
    syscall(&Syscall::Exit(code));
    unreachable!();
}

pub fn spawn(name: &str, arg: usize) -> usize {
    syscall(&Syscall::Spawn(name, arg))
}

pub fn socket() -> u64 {
    syscall(&Syscall::DgSocket) as u64
}

pub fn write(sn: u64, data: &[u8]) -> usize {
    syscall(&Syscall::DgWrite(sn, data))
}

pub fn async_read(sn: u64, data: &mut [u8]) -> u64 {
    syscall(&Syscall::DgRead(sn, data)) as u64
}

pub fn async_read_2(sn: u64, data: &mut [u8], task_id: TaskId) -> u64 {
    syscall(&Syscall::ReadAsync(sn, data, task_id)) as u64
}
