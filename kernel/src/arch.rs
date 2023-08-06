use core::arch::asm;
use core::time::Duration;

pub trait Arch {
    fn early_system_init(&self);
    unsafe fn early_cpu_init(&self);
    unsafe fn long_jump(&self, jump_to: usize) -> !;
    fn cpu_num(&self) -> u32;
    fn direct_map<T>(&self, ptr: *const T) -> *const T;
    fn direct_map_mut<T>(&self, ptr: *mut T) -> *mut T;
    fn enable_interrupts(&self);
    fn disable_interrupts(&self);
    fn send_ipi(&self, cpu_id: u8, vector: u8);
    fn sleep(&self, duration: Duration);
    fn sleep_forever(&self) -> !;
    fn sleep_forever_no_irq(&self) -> !;
}

// todo: config option
pub use crate::x86::Context;
pub use crate::x86::SERIAL;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InterruptSource {
    SerialPort,
}
