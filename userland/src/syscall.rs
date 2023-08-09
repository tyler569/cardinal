use core::arch::asm;
use cardinal3_interface::Syscall;

fn syscall(number: Syscall, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let result: usize;
    unsafe {
        asm!(
            "int 0x80",
            inout("rax") number as usize => result,
            in("r9") arg0,
            in("rcx") arg1,
            in("rdx") arg2,
            options(nostack)
        );
    }
    result
}

pub fn print(string: &str) -> usize {
    let args = cardinal3_interface::PrintArgs { data: string.as_bytes() };
    syscall(Syscall::Print, &args as *const _ as usize, 0, 0)
}