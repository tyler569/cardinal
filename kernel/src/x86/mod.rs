use crate::arch;
use crate::limine;
use core::arch::asm;
use core::cell::Cell;
use core::marker::Sync;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, Ordering};

mod acpi;
mod context;
mod cpu;
mod frame;
mod gdt;
mod idt;
mod interrupts;
mod ioapic;
mod lapic;
mod long_jump;
mod page;
mod pic;
mod pio;
mod serial;

pub use serial::SERIAL;

pub struct X86Arch {
    direct_map_offset: Cell<Option<usize>>,
}

impl X86Arch {
    const fn new() -> Self {
        Self {
            direct_map_offset: Cell::new(None),
        }
    }

    unsafe fn acpi_debug(&self) {
        let acpi_tables = acpi::init();
        let platform_info = ::acpi::PlatformInfo::new(&acpi_tables).unwrap();
        println!(
            "platform_info: {:?}",
            platform_info.processor_info.unwrap().application_processors
        );
        if let ::acpi::platform::interrupt::InterruptModel::Apic(apic) =
            platform_info.interrupt_model
        {
            println!("apic: {:#x?}", apic);
        }
    }
}

unsafe impl Sync for X86Arch {}

pub static X86ARCH: X86Arch = X86Arch::new();

use crate::arch::InterruptSource;
use crate::print::println;
use crate::x86::cpu::cpu_num;
pub use context::X86Context as Context;

static SYSTEM_INIT_DONE: AtomicBool = AtomicBool::new(false);

impl arch::Arch for X86Arch {
    fn early_system_init(&self) {
        if let Err(_) =
            SYSTEM_INIT_DONE.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        {
            return;
        }

        unsafe {
            cpu::system_init();
            idt::system_init();
            pic::remap_and_disable();

            self.early_cpu_init();

            let direct_map_offset = (**limine::HHDM.response.get()).offset;
            X86ARCH
                .direct_map_offset
                .set(Some(direct_map_offset as usize));

            // self.acpi_debug();
        }
    }

    unsafe fn early_cpu_init(&self) {
        cpu::use_();
        idt::load();
        lapic::init();
        lapic::start_timer();
    }

    unsafe fn long_jump(&self, jump_to: usize) -> ! {
        long_jump::long_jump(jump_to)
    }

    fn cpu_num(&self) -> u32 {
        cpu_num()
    }

    fn direct_map<T>(&self, ptr: *const T) -> *const T {
        let Some(offset) = X86ARCH.direct_map_offset.get() else {
            panic!("No higher-half offset found");
        };

        (ptr as usize + offset) as *const T
    }

    fn direct_map_mut<T>(&self, ptr: *mut T) -> *mut T {
        self.direct_map(ptr) as *mut T
    }

    fn enable_interrupts(&self) {
        unsafe {
            asm!("sti");
        }
    }

    fn disable_interrupts(&self) {
        unsafe {
            asm!("cli");
        }
    }

    fn send_ipi(&self, cpu_id: u8, vector: u8) {
        lapic::send_ipi(cpu_id, vector);
    }

    fn sleep(&self, duration: core::time::Duration) {
        // manually calibrated
        for _ in 0..duration.as_nanos() / 400 {
            unsafe { asm!("pause") };
        }
    }

    fn sleep_forever(&self) -> ! {
        loop {
            unsafe { asm!("hlt") };
        }
    }

    fn sleep_forever_no_irq(&self) -> ! {
        loop {
            unsafe { asm!("cli", "hlt") };
        }
    }
}

fn interrupt_map(source: InterruptSource) -> u8 {
    match source {
        InterruptSource::SerialPort => 0x4,
        _ => panic!("unimplemented interrupt source {:?}", source),
    }
}