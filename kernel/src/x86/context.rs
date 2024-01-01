use crate::arch::cpu_num;
use crate::per_cpu::PerCpu;
use crate::x86;
use core::arch::asm;
use core::fmt::Debug;
use core::fmt::Formatter;
use cardinal3_interface::SyscallReturn;

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
    pub fn new_user(ip: usize) -> Self {
        assert_ne!(ip, 0, "trying to create context to 0!");
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

    pub fn syscall_info(&self) -> &cardinal3_interface::Syscall {
        unsafe { &*(self.rax as *const cardinal3_interface::Syscall) }
    }

    pub fn set_syscall_return(&mut self, value: SyscallReturn) {
        match value {
            SyscallReturn::Complete(v) => {
                self.rax = 0;
                self.rdi = v;
            }
            SyscallReturn::NotComplete => {
                self.rax = 1;
            }
            SyscallReturn::Error(v) => {
                self.rax = 2;
                self.rdi = v as u64;
            }
        }
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
        writeln!(
            f,
            "ip {:016x} cs {:016x} fl {:016x}",
            self.ip, self.cs, self.flags,
        )?;
        write!(f, "cpu {}  pid {:?}", cpu_num(), PerCpu::running(),)?;
        Ok(())
    }
}

#[derive(Clone)]
#[repr(align(16))]
pub struct FpuContext([u8; 512]);

impl FpuContext {
    pub const fn new() -> Self {
        Self([0; 512])
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct Context {
    pub(super) frame: InterruptFrame,
    pub(super) fpu_context: FpuContext,
    pub(super) has_fpu_context: bool,
}

impl Context {
    pub fn new_user(user_ip: usize) -> Self {
        Self {
            frame: InterruptFrame::new_user(user_ip),
            fpu_context: FpuContext::new(),
            has_fpu_context: false,
        }
    }

    pub fn new(frame: &InterruptFrame) -> Self {
        let mut res = Self {
            frame: frame.clone(),
            fpu_context: FpuContext::new(),
            has_fpu_context: true,
        };
        unsafe {
            asm!("fxsave [{}]", in(reg) &mut res.fpu_context);
        }
        res
    }

    pub fn set_arg1(&mut self, arg1: u64) {
        self.frame.rdi = arg1;
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Context")
            .field("frame", &self.frame)
            .field("fpu_context", &"[...]")
            .finish()
    }
}
