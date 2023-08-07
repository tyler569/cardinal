use crate::limine;
use crate::print::println;
use core::arch::asm;
use core::marker::Sync;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::once::Once;

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

use crate::per_cpu::PerCpu;
pub use context::Context;
pub use cpu::{cpu_num, Cpu};
pub use serial::SERIAL;

static DIRECT_MAP_OFFSET: Once<usize> = Once::new();
static SYSTEM_INIT_DONE: AtomicBool = AtomicBool::new(false);

pub fn early_system_init() {
    if SYSTEM_INIT_DONE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    unsafe {
        idt::system_init();
        pic::remap_and_disable();

        early_cpu_init();

        let direct_map_offset = (**limine::HHDM.response.get()).offset;
        DIRECT_MAP_OFFSET.call_once(|| direct_map_offset as usize);

        let acpi_tables = acpi::init();
        let platform_info = ::acpi::PlatformInfo::new(&acpi_tables).unwrap();
        if let ::acpi::platform::interrupt::InterruptModel::Apic(apic) =
            platform_info.interrupt_model
        {
            ioapic::init(&apic);
        }

        ioapic::unmask_irq(4);

        // acpi_debug();
    }
}

pub unsafe fn early_cpu_init() {
    cpu::use_();
    idt::load();
    lapic::init();
    lapic::start_timer();
}

pub unsafe fn long_jump(jump_to: usize) -> ! {
    long_jump::long_jump(jump_to)
}

pub fn direct_map_offset() -> usize {
    let Some(offset) = DIRECT_MAP_OFFSET.get() else {
        panic!("Attempt to call direct_map without offset");
    };

    *offset
}

pub fn direct_map<T>(ptr: *const T) -> *const T {
    (ptr as usize + direct_map_offset()) as *const T
}

pub fn direct_map_mut<T>(ptr: *mut T) -> *mut T {
    direct_map(ptr) as *mut T
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti");
    }
}

pub fn disable_interrupts() {
    unsafe {
        asm!("cli");
    }
}

pub fn send_ipi(cpu_id: u8, vector: u8) {
    lapic::send_ipi(cpu_id, vector);
}

pub fn sleep(duration: core::time::Duration) {
    // manually calibrated
    for _ in 0..duration.as_nanos() / 400 {
        unsafe { asm!("pause") };
    }
}

pub fn sleep_forever() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

pub fn sleep_forever_no_irq() -> ! {
    loop {
        unsafe { asm!("cli", "hlt") };
    }
}

unsafe fn acpi_debug() {
    let acpi_tables = acpi::init();
    let platform_info = ::acpi::PlatformInfo::new(&acpi_tables).unwrap();
    println!(
        "platform_info: {:?}",
        platform_info.processor_info.unwrap().application_processors
    );
    if let ::acpi::platform::interrupt::InterruptModel::Apic(apic) = platform_info.interrupt_model {
        println!("apic: {:#x?}", apic);
    }
}
