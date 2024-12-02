use cardinal3_interface::SyscallContext;
use core::arch::asm;
use crate::executor;

pub(crate) fn syscall_future(args: &mut SyscallContext, task_id: u64) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") args as *const _,
            in("rdi") task_id,
            options(nostack)
        );
    }
}

pub fn print(_string: impl AsRef<str>) {
//     executor::dispatch_syscall(&Syscall::Print(string.as_ref()));
}

pub fn exit(_code: u64) -> ! {
//     executor::dispatch_syscall(&Syscall::Exit(code));
    unreachable!()
}
