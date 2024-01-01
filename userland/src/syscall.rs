use cardinal3_interface::{Error, Syscall, SyscallReturn};
use num_traits::FromPrimitive;
use core::arch::asm;

fn syscall(args: &Syscall) -> SyscallReturn {
    let return_type: u64;
    let return_value: u64;
    unsafe {
        asm!(
            "int 0x80",
            inout("rax") args as *const _ as usize => return_type,
            out("rdi") return_value,
            options(nostack)
        );
    }
    match return_type {
        0 => SyscallReturn::Complete(return_value),
        1 => SyscallReturn::NotComplete,
        2 => SyscallReturn::Error(Error::from_u64(return_value).expect("Invalid error return")),
        _ => panic!("Invalid syscall return"),
    }
}

pub fn println(string: &str) {
    syscall(&Syscall::Println(string));
}

pub fn exit(code: u64) -> ! {
    syscall(&Syscall::Exit(code));
    unreachable!();
}

// pub fn spawn(name: &str, arg: usize) -> usize {
//     syscall(&Syscall::Spawn(name, arg))
// }
//
// pub fn socket() -> u64 {
//     syscall(&Syscall::DgSocket) as u64
// }
//
// pub fn write(sn: u64, data: &[u8]) -> usize {
//     syscall(&Syscall::DgWrite(sn, data))
// }
//
// pub fn async_read(sn: u64, data: &mut [u8]) -> u64 {
//     syscall(&Syscall::DgRead(sn, data)) as u64
// }
