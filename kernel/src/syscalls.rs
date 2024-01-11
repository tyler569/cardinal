use core::time::Duration;
use crate::net::{socket, Socket};
use crate::per_cpu::PerCpu;
use crate::print::print;
use crate::println;
use crate::{arch, process};
use cardinal3_interface::{Error, Syscall, SyscallReturn};
use crate::executor::sleep::sleep;

pub fn handle_syscall(frame: &mut arch::InterruptFrame) {
    let syscall = frame.syscall_info();
    let task_id = frame.task_id();
    let tasks_to_wake = frame.tasks_to_wake();
    let pid = PerCpu::running().expect("syscall without running process!");

    match syscall {
        Syscall::Print(arg) => print!("{}", arg),
        _ => println!(
            "[cpu:{} pid:{} syscall:{:?} tsc:{}]",
            arch::cpu_num(),
            pid,
            syscall,
            arch::rdtsc(),
        ),
    }

    let result = match syscall {
        Syscall::Print(_) => SyscallReturn::Complete(0),
        Syscall::Exit(code) => {
            process::exit(*code);
            SyscallReturn::Complete(0)
        }
        Syscall::Spawn(name, arg) => SyscallReturn::Complete(process::spawn(name, *arg)),
        Syscall::DgSocket => SyscallReturn::Complete(Socket::new()),
        Syscall::DgRead(sn, buf) => socket::read(*sn, buf),
        Syscall::DgWrite(sn, buf) => socket::write(*sn, buf),
        &Syscall::Sleep(usec) => {
            PerCpu::executor_mut().spawn(async move {
                sleep(Duration::from_micros(usec)).await;
                process::schedule_wakeup(pid, task_id);
            });
            SyscallReturn::NotComplete
        }
        Syscall::Yield => {
            process::with(pid, |proc| {
                proc.wait(frame);
            });
            SyscallReturn::Complete(0)
        }
        _ => SyscallReturn::Error(Error::InvalidSyscall),
    };

    let count = process::with(pid, |proc| {
        let count = proc.drain_tasks_to_wake(tasks_to_wake);
        if count > 0 {
            // in case this was a call to Yield and we already have work to do
            proc.unwait();
        }
        count
    }).unwrap();

    frame.set_syscall_return(result);
    frame.set_tasks_to_wake_count(count);
}
