use crate::arch::InterruptFrame;
use crate::per_cpu::PerCpu;
use crate::print::{print, println};
use crate::process::Process;
use crate::{arch, elf_data, process};
use cardinal3_interface::Syscall;

pub fn handle_syscall(frame: &mut InterruptFrame) {
    let syscall = frame.syscall_info();
    let pid = PerCpu::running().unwrap_or(0);

    println!(
        "[cpu:{} pid:{} syscall:{:?}]",
        arch::cpu_num(),
        pid,
        syscall
    );

    match syscall {
        &Syscall::Print(string) => {
            // print!("{}", pid, string);
            frame.set_syscall_return(string.len());
        }
        &Syscall::Exit(code) => unsafe {
            let Some(pid) = PerCpu::running() else {
                panic!("No running process");
            };
            process::ALL.lock().get_mut(&pid).unwrap().exit_code = Some(code);
        },
        &Syscall::Spawn(_name, arg) => unsafe {
            let pid = Process::new(&*elf_data(), arg);
            process::schedule_pid(pid);
        },
    }
}
