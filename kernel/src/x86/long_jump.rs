use core::arch::asm;
use crate::x86::cpu;

pub unsafe fn long_jump(jump_to: usize) -> ! {
    let new_stack = cpu::kernel_stack();
    asm!(
        "push 0", // ss
        "push {new_stack}", // sp
        "pushf", // rflags
        "push 0x8", // cs
        "push {jump_to}", // ip
        "iretq",
        jump_to = in(reg) jump_to,
        new_stack = in(reg) new_stack,
        options(noreturn)
    )
}
