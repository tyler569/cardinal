use crate::x86::frame::InterruptFrame;

pub struct Context {
    frame: InterruptFrame,
    fpu_context: [u8; 512],
}
