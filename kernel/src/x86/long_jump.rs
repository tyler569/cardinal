use crate::x86::cpu;
use core::arch::asm;

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

pub unsafe fn long_jump_usermode(jump_to: usize, stack: usize) -> ! {
    asm!(
        // "mov ax, 0",
        // "mov ds, ax",
        // "mov es, ax",
        // "mov fs, ax",
        // "mov gs, ax",
        "mov rax, 0",
        "mov rbx, 0",
        "mov rcx, 0",
        "mov rdx, 0",
        "mov r8, 0",
        "mov r9, 0",
        "mov r10, 0",
        "mov r11, 0",
        "mov r12, 0",
        "mov r13, 0",
        "mov r14, 0",
        "mov r15, 0",

        "push 0x23", // ss
        "push rsi", // sp
        "push 0x200", // rflags (IF)
        "push 0x1b", // cs
        "push rdi", // ip

        "mov rbp, rsi", // set rbp to rsp
        "mov rdi, 0",
        "mov rsi, 0",

        "iretq",
        in("rdi") jump_to,
        in("rsi") stack,
        options(noreturn)
    )
}
