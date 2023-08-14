use crate::x86;
use core::fmt::Formatter;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct InterruptFrame {
    pub(super) ds: u64,
    pub(super) r15: u64,
    pub(super) r14: u64,
    pub(super) r13: u64,
    pub(super) r12: u64,
    pub(super) r11: u64,
    pub(super) r10: u64,
    pub(super) r9: u64,
    pub(super) r8: u64,
    pub(super) rbp: u64,
    pub(super) rdi: u64,
    pub(super) rsi: u64,
    pub(super) rdx: u64,
    pub(super) rcx: u64,
    pub(super) rbx: u64,
    pub(super) rax: u64,
    pub(super) interrupt_number: u64,
    pub(super) error_code: u64,
    pub(super) ip: u64,
    pub(super) cs: u64,
    pub(super) flags: u64,
    pub(super) user_sp: u64,
    pub(super) ss: u64,
}

impl InterruptFrame {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_user(ip: usize) -> Self {
        Self {
            r12: 0x1234,
            ip: ip as u64,
            cs: 0x1b,
            flags: 0x200,
            ss: 0x23,
            user_sp: x86::USER_STACK_TOP as u64,
            ..Default::default()
        }
    }

    pub fn interrupt_number(&self) -> u64 {
        self.interrupt_number
    }

    pub fn syscall_info(&self) -> &cardinal3_interface::Syscall {
        unsafe { &*(self.rax as *const cardinal3_interface::Syscall) }
    }

    pub fn set_syscall_return(&mut self, value: usize) {
        self.rax = value as u64;
    }

    pub fn ip(&self) -> usize {
        self.ip as usize
    }
}

impl core::fmt::Display for InterruptFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "ax {:016x} bx {:016x} cx {:016x} dx {:016x}",
            self.rax, self.rbx, self.rcx, self.rdx
        )?;
        writeln!(
            f,
            "sp {:016x} bp {:016x} si {:016x} di {:016x}",
            self.user_sp, self.rbp, self.rsi, self.rdi
        )?;
        writeln!(
            f,
            " 8 {:016x}  9 {:016x} 10 {:016x} 11 {:016x}",
            self.r8, self.r9, self.r10, self.r11
        )?;
        writeln!(
            f,
            "12 {:016x} 13 {:016x} 14 {:016x} 15 {:016x}",
            self.r12, self.r13, self.r14, self.r15
        )?;
        write!(
            f,
            "ip {:016x} cs {:016x} fl {:016x}",
            self.ip, self.cs, self.flags
        )?;
        Ok(())
    }
}
