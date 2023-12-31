use crate::net::{socket, Socket};
use crate::per_cpu::PerCpu;
use crate::println;
use crate::{arch, process};
use cardinal3_interface::{Error, Syscall};

pub fn handle_syscall(frame: &mut arch::InterruptFrame) {
    let syscall = frame.syscall_info();
    let pid = PerCpu::running().unwrap_or(0);

    println!(
        "[cpu:{} pid:{} syscall:{:?}]",
        arch::cpu_num(),
        pid,
        syscall
    );

    let result = match syscall {
        &Syscall::Println(_) => Ok(0),
        &Syscall::Exit(code) => Ok(process::exit(code)),
        &Syscall::Spawn(name, arg) => Ok(process::spawn(name, arg)),
        &Syscall::DgSocket => Ok(Socket::new()),
        &Syscall::DgRead(sn, buf) => socket::read(sn, buf),
        &Syscall::DgWrite(sn, buf) => socket::write(sn, buf),
        &Syscall::ReadAsync(..) => Ok(0),
        _ => {
            println!("Unknown syscall");
            Err(Error::EINVAL)
        }
    };

    match result {
        Ok(value) => frame.set_syscall_return(value as usize),
        Err(err) => frame.set_syscall_return(err.return_value()),
    }
}
