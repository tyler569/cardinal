use crate::arch::InterruptFrame;
use crate::net::{Packet, Socket};
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

    match syscall {
        &Syscall::Println(string) => {}
        &Syscall::Exit(code) => unsafe {
            let Some(pid) = PerCpu::running() else {
                panic!("No running process");
            };
            process::ALL.lock().get_mut(&pid).unwrap().exit_code = Some(code);
        }
        &Syscall::Spawn(_name, arg) => unsafe {
            let pid = Process::new(&*elf_data(), arg);
            process::schedule_pid(pid);
        }
        &Syscall::DgSocket => {
            let socket = Socket::new();
            frame.set_syscall_return(socket as usize);
        }
        &Syscall::DgRead(sn, buf) => {
            let task = {
                let binding = crate::net::socket::ALL.lock();
                let socket = binding.get(&sn).unwrap();
                socket.read(buf)
            };
            let pid = PerCpu::running().unwrap();
            crate::executor::spawn(async move {
                task.await;
                process::ALL
                    .lock()
                    .get_mut(&pid)
                    .map(|proc| proc.pending_signals |= 0x01);
                println!("read completed");
            });
        }
        &Syscall::DgWrite(sn, buf) => {
            let binding = crate::net::socket::ALL.lock();
            let socket = binding.get(&sn).unwrap();
            let packet = Packet::new(buf);
            socket.write(packet);
        }
        &Syscall::ReadAsync(..) => {}
        _ => {
            println!("Unknown syscall");
            frame.set_syscall_return(Error::EINVAL as usize);
        }
    }
}
