use crate::net::{socket, Socket};
use crate::per_cpu::PerCpu;
use crate::println;
use crate::{arch, process};
use cardinal3_interface::{Error, Syscall, SyscallReturn};

pub fn handle_syscall(frame: &mut arch::InterruptFrame) {
    let syscall = frame.syscall_info();
    let _task_id = frame.task_id();
    let _tasks_to_wake = frame.tasks_to_wake();
    let pid = PerCpu::running().unwrap_or(0);

    println!(
        "[cpu:{} pid:{} syscall:{:?}]",
        arch::cpu_num(),
        pid,
        syscall
    );

    let result = match syscall {
        Syscall::Println(_) => SyscallReturn::Complete(0),
        Syscall::Exit(code) => {
            process::exit(*code);
            SyscallReturn::Complete(0)
        },
        Syscall::Spawn(name, arg) => {
            SyscallReturn::Complete(process::spawn(name, *arg))
        },
        Syscall::DgSocket => SyscallReturn::Complete(Socket::new()),
        Syscall::DgRead(sn, buf) => socket::read(*sn, buf),
        Syscall::DgWrite(sn, buf) => socket::write(*sn, buf),
        _ => SyscallReturn::Error(Error::InvalidSyscall),
    };

    frame.set_syscall_return(result);
    frame.set_tasks_to_wake_count(0);
}
