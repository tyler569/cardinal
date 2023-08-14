use crate::arch;
use crate::arch::InterruptFrame;
use crate::print::{print, println};
use cardinal3_interface::Syscall;
use crate::per_cpu::PerCpu;

pub fn handle_syscall(frame: &mut InterruptFrame) {
    let syscall = frame.syscall_info();

    // println!("syscall: {:?} {:?}", syscall as *const _, syscall);

    match syscall {
        Syscall::Print(string) => {
            print!("{}", string);
            frame.set_syscall_return(0);
        }
        Syscall::Exit(code) => unsafe {
            println!("Process exited with code {}", code);
            PerCpu::get_mut().running.unwrap().as_mut().exit_code = Some(*code);
        }
    }
}
