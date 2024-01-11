use crate::ipi::handle_ipi_irq;
use crate::per_cpu::PerCpu;
use crate::println;
use crate::process::ProcessDisposition;
use crate::x86::context::InterruptFrame;
use crate::x86::cpu::cpu_num;
use crate::x86::{cpu, lapic, print_backtrace_from_frame, sleep_forever_no_irq, SERIAL};
use crate::{arch, executor, process, syscalls};
use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

static INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);

#[no_mangle]
unsafe extern "C" fn rs_interrupt_shim(frame: *mut InterruptFrame) {
    let frame = &mut *frame;

    let bp: usize;
    asm!("mov {}, rbp", out(reg) bp, options(nostack));
    assert_eq!(bp & 0xf, 0, "stack not aligned to 16 bytes");

    let _count = INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
    let from_usermode = frame.cs & 0x03 == 0x03;

    match frame.interrupt_number {
        1 => handle_debug(frame),
        3 => handle_breakpoint(frame),
        14 => handle_page_fault(frame),
        32..=47 => handle_irq(frame),
        128 => handle_syscall(frame),
        129 => handle_ipi(frame),
        130 => handle_ipi_panic(frame),
        _ => unexpected_interrupt(frame),
    }

    let old_pid = PerCpu::running();
    let old_vm_root = old_pid.and_then(|pid| process::with(pid, |proc| proc.vm_root()));

    if from_usermode {
        let pid = old_pid.expect("Interrupt from usermode with no process on CPU");
        let should_run = process::with(pid, |p| {
            p.set_context(frame);
            p.set_on_cpu(None);
            p.should_run()
        })
        .expect("Interrupt from usermode with process that no longer exists");
        // println!("({:#018x}) <>", frame.ip);
        // print_backtrace_from_frame(frame);
        // println!("----");
        match should_run {
            ProcessDisposition::MayContinue => {}
            ProcessDisposition::TimesUp => {
                process::maybe_run_usermode_program(true);
            }
            ProcessDisposition::NotNow => {
                process::maybe_run_usermode_program(false);
                arch::sleep_forever();
            }
            ProcessDisposition::NeverAgain => {
                executor::spawn(async move {
                    process::remove(pid);
                });
                process::maybe_run_usermode_program(false);
                arch::sleep_forever();
            }
        }

        arch::load_tree(old_vm_root.expect("Returning to process with no vm_root"));
        process::with(pid, |p| p.set_on_cpu(Some(cpu_num())));
    } else {
        process::maybe_run_usermode_program(false);
    }
    assert_ne!(frame.ip, 0, "Returning from interrupt to IP 0");
}

fn handle_breakpoint(frame: &InterruptFrame) {
    println!("break point");
    println!("{}", frame);
}

fn handle_debug(frame: &InterruptFrame) {
    println!("step ip: {:x}", frame.ip);
}

fn handle_page_fault(frame: &InterruptFrame) {
    report_page_fault(frame, frame.error_code, cpu::cr2());
    panic!("Unhandled page fault\n{}", frame);
}

fn report_page_fault(frame: &InterruptFrame, error_code: u64, _fault_addr: u64) {
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
    );

    println!("access came from here:");
    print_backtrace_from_frame(frame);
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

fn handle_timer(_frame: &InterruptFrame) {
    PerCpu::timer_mut().tick();
    PerCpu::executor_mut().do_work();
}

fn handle_serial(_frame: &InterruptFrame) {
    unsafe { SERIAL.handle_interrupt() };
}

fn handle_syscall(frame: &mut InterruptFrame) {
    syscalls::handle_syscall(frame);
}

fn handle_ipi(_frame: &InterruptFrame) {
    handle_ipi_irq();

    lapic::eoi();
}

fn handle_ipi_panic(_frame: &InterruptFrame) {
    // println!("CPU {} stopping due to panic on another CPU", cpu_num());
    sleep_forever_no_irq();
}

fn unexpected_interrupt(frame: &InterruptFrame) {
    println!(
        "CPU {} Unhandled {}",
        cpu_num(),
        INTERRUPT_INFO[frame.interrupt_number as usize].name
    );
    panic!(
        "Unhandled unexpected interrupt {:x} ({})\n{}",
        frame.interrupt_number, frame.interrupt_number, frame
    );
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
