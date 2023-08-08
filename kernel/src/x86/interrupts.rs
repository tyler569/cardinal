use crate::per_cpu::PerCpu;
use crate::print::println;
use crate::x86;
use crate::x86::cpu::cpu_num;
use crate::x86::frame::InterruptFrame;
use crate::x86::{lapic, SERIAL};
use core::arch::asm;
use core::cell::UnsafeCell;
use core::ops::Deref;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Lazy;

pub unsafe fn enable_interrupts() {
    asm!("cli");
}

pub unsafe fn disable_interrupts() {
    asm!("sti");
}

#[no_mangle]
unsafe extern "C" fn rs_interrupt_shim(frame: *mut InterruptFrame) {
    match (*frame).interrupt_number {
        3 => handle_breakpoint(&mut *frame),
        32..=47 => handle_irq(&mut *frame),
        128 => handle_syscall(&mut *frame),
        129 => handle_ipi(&mut *frame),
        130 => handle_ipi_panic(&mut *frame),
        _ => unexpected_interrupt(&*frame),
    }
}

fn handle_breakpoint(frame: &mut InterruptFrame) {
    println!("break point");
    println!("{}", frame);
}

fn handle_irq(frame: &mut InterruptFrame) {
    let irq_num = frame.interrupt_number - 32;

    match irq_num {
        0 => handle_timer(frame),
        4 => handle_serial(frame),
        _ => println!("CPU {} Unhandled IRQ {}", cpu_num(), irq_num),
    }

    lapic::eoi();
}

fn handle_timer(frame: &mut InterruptFrame) {
    let cpu = cpu_num() as usize;
    PerCpu::get_mut().timer.tick();
}

fn handle_serial(frame: &mut InterruptFrame) {
    unsafe { SERIAL.handle_interrupt() };
}

fn handle_syscall(frame: &mut InterruptFrame) {
    println!("syscall: {}", frame.rax);
}

fn handle_ipi(frame: &mut InterruptFrame) {
    println!("CPU {} IPI", cpu_num());

    lapic::eoi();
}

fn handle_ipi_panic(frame: &mut InterruptFrame) {
    println!("CPU {} stopping due to panic on another CPU", cpu_num());
    x86::sleep_forever_no_irq();
}

fn unexpected_interrupt(frame: &InterruptFrame) {
    println!(
        "CPU {} Unhandled {}",
        cpu_num(),
        INTERRUPT_INFO[frame.interrupt_number as usize].name
    );
    println!("{}", frame);
    panic!();
}

#[derive(Debug)]
pub struct InterruptInfo {
    pub name: &'static str,
    pub short: &'static str,
}

pub const INTERRUPT_INFO: &[InterruptInfo] = &[
    InterruptInfo {
        name: "Divide Error",
        short: "#DE",
    },
    InterruptInfo {
        name: "Debug Exception",
        short: "#DB",
    },
    InterruptInfo {
        name: "Non-Maskable Interrupt",
        short: "NMI",
    },
    InterruptInfo {
        name: "Breakpoint",
        short: "#BP",
    },
    InterruptInfo {
        name: "Overflow",
        short: "#OF",
    },
    InterruptInfo {
        name: "Bound Range Exceeded",
        short: "#BR",
    },
    InterruptInfo {
        name: "Invalid Opcode",
        short: "#UD",
    },
    InterruptInfo {
        name: "Device Not Available",
        short: "#NM",
    },
    InterruptInfo {
        name: "Double Fault",
        short: "#DF",
    },
    InterruptInfo {
        name: "Coprocessor Segment Overrun",
        short: "<none>",
    },
    InterruptInfo {
        name: "Invalid TSS",
        short: "#TS",
    },
    InterruptInfo {
        name: "Segment Not Present",
        short: "#NP",
    },
    InterruptInfo {
        name: "Stack-Segment Fault",
        short: "#SS",
    },
    InterruptInfo {
        name: "General Protection Fault",
        short: "#GP",
    },
    InterruptInfo {
        name: "Page Fault",
        short: "#PF",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "x87 Floating-Point Exception",
        short: "#MF",
    },
    InterruptInfo {
        name: "Alignment Check",
        short: "#AC",
    },
    InterruptInfo {
        name: "Machine Check",
        short: "#MC",
    },
    InterruptInfo {
        name: "SIMD Floating-Point Exception",
        short: "#XM",
    },
    InterruptInfo {
        name: "Virtualization Exception",
        short: "#VE",
    },
    InterruptInfo {
        name: "Control Protection Exception",
        short: "#CP",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
    InterruptInfo {
        name: "Security Exception",
        short: "#SX",
    },
    InterruptInfo {
        name: "Reserved",
        short: "<reserved>",
    },
];
