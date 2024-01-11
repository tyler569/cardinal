use cardinal3_interface::{Error, Syscall, SyscallReturn};
use core::arch::asm;
use num_traits::FromPrimitive;

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
            2 => SyscallReturn::Error(Error::from_u64(return_value).expect("Invalid error return")),
            _ => panic!("Invalid syscall return"),
        },
        wake_count,
    )
}

pub fn syscall_simple(args: &Syscall) -> SyscallReturn {
    syscall_future(args, 0, &mut [0; 0]).0
}

pub fn print(string: impl AsRef<str>) {
    syscall_simple(&Syscall::Print(string.as_ref()));
}

pub fn exit(code: u64) -> ! {
    syscall_simple(&Syscall::Exit(code));
    unreachable!();
}
