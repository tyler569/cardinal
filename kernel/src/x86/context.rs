use crate::arch::cpu_num;
use crate::per_cpu::PerCpu;
use crate::x86;
use bitflags::bitflags;
use cardinal3_interface::SyscallReturn;
use core::arch::asm;
use core::fmt::Debug;
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

bitflags! {
    pub struct X86Flags: u64 {
        const CARRY = 1 << 0;
        const PARITY = 1 << 2;
        const ADJUST = 1 << 4;
        const ZERO = 1 << 6;
        const SIGN = 1 << 7;
        const TRAP = 1 << 8;
        const INTERRUPT = 1 << 9;
        const DIRECTION = 1 << 10;
        const OVERFLOW = 1 << 11;
        const IOPL0 = 1 << 12;
        const IOPL1 = 1 << 13;
        const NESTED_TASK = 1 << 14;
        const RESUME = 1 << 16;
        const VIRTUAL_8086 = 1 << 17;
        const ALIGNMENT_CHECK = 1 << 18;
        const VIRTUAL_INTERRUPT = 1 << 19;
        const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
        const ID = 1 << 21;
    }
}

pub const DEFAULT_FLAGS: X86Flags = X86Flags::INTERRUPT;

impl InterruptFrame {
    pub fn new_user(ip: usize) -> Self {
        assert_ne!(ip, 0, "trying to create context to 0!");
        Self {
            r12: 0x1234,
            ip: ip as u64,
            cs: 0x1b,
            flags: DEFAULT_FLAGS.bits(),
            ss: 0x23,
            user_sp: x86::USER_STACK_TOP as u64,
            ..Default::default()
        }
    }

    pub fn syscall_context(&self) -> &mut cardinal3_interface::SyscallContext {
        unsafe { &mut *(self.rax as *mut cardinal3_interface::SyscallContext) }
    }

    pub fn task_id(&self) -> u64 {
        self.rdi
    }

    pub fn tasks_to_wake(&self) -> &'static mut [u64] {
        unsafe { core::slice::from_raw_parts_mut(self.rsi as *mut u64, self.rdx as usize) }
    }

    pub fn set_tasks_to_wake_count(&mut self, count: usize) {
        self.rdx = count as u64;
    }
}

impl core::fmt::Display for InterruptFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "frame at {:p}, cpu {}, pid {:?}",
            self,
            cpu_num(),
            PerCpu::running()
        )?;
        writeln!(
            f,
            " ax {:016x} bx {:016x} cx {:016x} dx {:016x}",
            self.rax, self.rbx, self.rcx, self.rdx
        )?;
        writeln!(
            f,
            " sp {:016x} bp {:016x} si {:016x} di {:016x}",
            self.user_sp, self.rbp, self.rsi, self.rdi
        )?;
        writeln!(
            f,
            "  8 {:016x}  9 {:016x} 10 {:016x} 11 {:016x}",
            self.r8, self.r9, self.r10, self.r11
        )?;
        writeln!(
            f,
            " 12 {:016x} 13 {:016x} 14 {:016x} 15 {:016x}",
            self.r12, self.r13, self.r14, self.r15
        )?;
        write!(
            f,
            " ip {:016x} cs {:016x} fl {:016x}",
            self.ip, self.cs, self.flags,
        )?;
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
    pub(crate) frame: InterruptFrame,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Context")
            .field("frame", &self.frame)
            .field("fpu_context", &"[...]")
            .finish()
    }
}
