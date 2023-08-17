use cardinal3_interface::Syscall;
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

pub fn print(string: &str) -> usize {
    syscall(&Syscall::Print(string))
}

pub fn exit(code: u32) -> ! {
    syscall(&Syscall::Exit(code));
    unreachable!();
}

pub fn spawn(name: &str, arg: usize) -> usize {
    syscall(&Syscall::Spawn(name, arg))
}
