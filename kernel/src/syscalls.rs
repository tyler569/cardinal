use crate::{arch, elf_data, process};
use crate::arch::InterruptFrame;
use crate::print::{print, println};
use cardinal3_interface::Syscall;
use crate::per_cpu::PerCpu;
use crate::process::Process;

pub fn handle_syscall(frame: &mut InterruptFrame) {
    let syscall = frame.syscall_info();
    let pid = PerCpu::running().map(|p| p.id).unwrap_or(0);

    // println!("syscall: {:?} {:?}", syscall as *const _, syscall);

    match syscall {
        Syscall::Print(string) => {
            print!("{}: {}", pid, string);
            frame.set_syscall_return(0);
        }
        &Syscall::Exit(code) => unsafe {
            // println!("{}: exit {}", pid, code);
            let Some(proc) = PerCpu::running() else {
                panic!("No running process");
            };
            proc.exit_code = Some(code);
        }
        &Syscall::Spawn(_name, arg) => unsafe {
            let pid = Process::new(&*elf_data(), arg);
            process::schedule_pid(pid);
        }
    }
}
