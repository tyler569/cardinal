use core::arch::asm;
use crate::x86::frame::InterruptFrame;
use core::fmt::Debug;
use crate::print::println;

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

    pub fn ip(&self) -> usize {
        self.frame.ip()
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
