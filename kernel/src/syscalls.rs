use crate::arch::InterruptFrame;
use crate::net::{socket, Packet, Socket};
use crate::per_cpu::PerCpu;
use crate::print::{print, println};
use crate::process::Process;
use crate::{arch, elf_data, process};
use cardinal3_interface::{Error, Syscall};

pub fn handle_syscall(frame: &mut InterruptFrame) {
    let syscall = frame.syscall_info();
    let pid = PerCpu::running().unwrap_or(0);

    println!(
        "[cpu:{} pid:{} syscall:{:?}]",
        arch::cpu_num(),
        pid,
        syscall
    );

    let result = match syscall {
        &Syscall::Println(string) => 0,
        &Syscall::Exit(code) => process::exit(code),
        &Syscall::Spawn(name, arg) => process::spawn(name, arg),
        &Syscall::DgSocket => Socket::new(),
        &Syscall::DgRead(sn, buf) => socket::read(sn, buf),
        &Syscall::DgWrite(sn, buf) => socket::write(sn, buf),
        &Syscall::ReadAsync(..) => 0,
        _ => {
            println!("Unknown syscall");
            Error::EINVAL as u64
        }
    };

    frame.set_syscall_return(result as usize);
}
