use crate::x86::frame::InterruptFrame;
use core::fmt::Debug;

pub struct Context {
    frame: InterruptFrame,
    fpu_context: [u8; 512],
}

impl Context {
    pub fn new_user(user_ip: usize) -> Self {
        Self {
            frame: InterruptFrame::new_user(user_ip),
            fpu_context: [0; 512],
        }
    }

    pub fn ip(&self) -> usize {
        self.frame.ip()
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
