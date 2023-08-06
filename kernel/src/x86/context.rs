use crate::x86::frame::InterruptFrame;

pub struct X86Context {
    frame: InterruptFrame,
    fpu_context: [u8; 512],
}
