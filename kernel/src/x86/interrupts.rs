use crate::per_cpu::PerCpu;
use crate::print::{print, println};
use crate::process::Process;
use crate::x86::context::{Context, InterruptFrame};
use crate::x86::cpu::cpu_num;
use crate::x86::{cpu, lapic, print_backtrace_from, sleep_forever_no_irq, SERIAL};
use crate::{arch, executor, process, syscalls};
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

static INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);

#[no_mangle]
unsafe extern "C" fn rs_interrupt_shim(frame: *mut InterruptFrame) {
    let frame = &mut *frame;

    let bp: usize;
    asm!("mov {}, rbp", out(reg) bp, options(nostack));
    assert_eq!(bp & 0xf, 0, "stack not aligned to 16 bytes");

    let count = INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
    let from_usermode = frame.cs & 0x03 == 0x03;

    match (*frame).interrupt_number {
        3 => handle_breakpoint(frame),
        14 => handle_page_fault(frame),
        32..=47 => handle_irq(frame),
        128 => handle_syscall(frame),
        129 => handle_ipi(frame),
        130 => handle_ipi_panic(frame),
        _ => unexpected_interrupt(frame),
    }

    let mut wants_continue = true;
    let old_pid = PerCpu::running();

    if from_usermode {
        let pid = old_pid.expect("No running usermode process");
        let mut binding = process::ALL.lock();
        if let Some(mut proc) = binding.get_mut(&pid) {
            proc.context = Context::new(frame);
            if proc.exit_code.is_some() {
                wants_continue = false;
                executor::spawn(async move {
                    process::ALL.lock().remove(&pid);
                })
            } else {
                if proc.time_expired() {
                    process::schedule_pid(pid);
                } else {
                    arch::load_tree(proc.vm_root);
                    assert_ne!(frame.ip, 0, "trying to return to 0!");
                    return;
                }
            }
        }
    }

    process::maybe_run_usermode_program();

    if wants_continue {
        if from_usermode {
            let pid = PerCpu::running().unwrap();
            let Some(vm_root) = process::with(pid, |p| p.vm_root) else {
                PerCpu::set_running(None);
                arch::sleep_forever()
            };
            arch::load_tree(vm_root);
        }
        assert_ne!(frame.ip, 0, "trying to return to 0!");
        return;
    } else {
        PerCpu::set_running(None);
        arch::sleep_forever()
    }
}

fn handle_breakpoint(frame: &mut InterruptFrame) {
    println!("break point");
    println!("{}", frame);
}

fn handle_page_fault(frame: &mut InterruptFrame) {
    report_page_fault(frame.error_code, cpu::cr2());
    println!("{}", frame);
    panic!();
}

fn report_page_fault(error_code: u64, fault_addr: u64) {
    if error_code & !0x1F != 0 {
        println!(
            "page fault caused by unknown condition (code: {:#x})",
            error_code
        );
        return;
    }
    if error_code & 0x8 != 0 {
        println!("page fault caused by writing to a reserved field");
        return;
    }
    let reason = if error_code & 0x1 != 0 {
        "protection violation"
    } else {
        "non-present page"
    };
    let rw = if error_code & 0x2 != 0 {
        "writing"
    } else {
        "reading"
    };
    let mode = if error_code & 0x4 != 0 {
        "user"
    } else {
        "kernel"
    };
    let typ = if error_code & 0x10 != 0 {
        "instruction"
    } else {
        "data"
    };

    println!(
        "page fault {} {}:{:#x} because {} from {} mode.",
        rw,
        typ,
        cpu::cr2(),
        reason,
        mode
    )
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
    PerCpu::get_mut().executor.do_work();
}

fn handle_serial(frame: &mut InterruptFrame) {
    unsafe { SERIAL.handle_interrupt() };
}

fn handle_syscall(frame: &mut InterruptFrame) {
    syscalls::handle_syscall(frame);
}

fn handle_ipi(frame: &mut InterruptFrame) {
    println!("CPU {} IPI", cpu_num());

    lapic::eoi();
}

fn handle_ipi_panic(frame: &mut InterruptFrame) {
    println!("CPU {} stopping due to panic on another CPU", cpu_num());
    sleep_forever_no_irq();
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
