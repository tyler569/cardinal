use crate::arch;
use crate::arch::InterruptFrame;
use crate::print::{print, println};
use cardinal3_interface::Syscall;

pub fn handle_syscall(frame: &mut InterruptFrame) {
    let syscall = frame.syscall_info();

    println!("syscall: {:?} {:?}", syscall as *const _, syscall);

    match syscall {
        Syscall::Print(string) => {
            print!("{}", string);
            frame.set_syscall_return(0);
        }
        Syscall::Exit(code) => {
            println!("Process exited with code {}", code);
            frame.set_syscall_return(0);
            arch::sleep_forever_no_irq();
        }
    }
}
