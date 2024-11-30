use cardinal3_interface::{Syscall, SyscallReturn};
use core::arch::asm;
use crate::executor;

pub(crate) fn syscall_future(
    args: &Syscall,
    task_id: u64,
    tasks_to_wake: &mut [u64],
) -> (SyscallReturn, usize) {
    let return_type: u64;
    let return_value: u64;
    let wake_count: usize;
    unsafe {
        asm!(
            "int 0x80",
            inout("rax") args as *const _ => return_type,
            inout("rdi") task_id => return_value,
            in("rsi") tasks_to_wake.as_mut_ptr(),
            inout("rdx") tasks_to_wake.len() => wake_count,
            options(nostack)
        );
    }
    (
        match return_type {
            0 => SyscallReturn::Complete(return_value),
            1 => SyscallReturn::NotComplete,
            2 => todo!("Syscall error not implemented"),
            _ => panic!("Invalid syscall return"),
        },
        wake_count,
    )
}

pub fn print(string: impl AsRef<str>) {
    executor::dispatch_syscall(&Syscall::Print(string.as_ref()));
}

pub fn exit(code: u64) -> ! {
    executor::dispatch_syscall(&Syscall::Exit(code));
    unreachable!();
}
