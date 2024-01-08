use crate::limine;
use crate::pci::PciAddress;
use crate::print::println;
use core::arch::asm;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Lazy;

mod acpi;
mod context;
mod cpu;
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

pub use context::{Context, InterruptFrame};
pub use cpu::{cpu_num, Cpu};
pub use long_jump::{long_jump_context, long_jump_cs};
pub use page::{
    free_tree, load_tree, map_in_table, new_tree, physical_address, PageTable,
};
pub use serial::SERIAL;

static DIRECT_MAP_OFFSET: Lazy<usize> =
    Lazy::new(|| unsafe { (**limine::HHDM.response.get()).offset } as usize);
static SYSTEM_INIT_DONE: AtomicBool = AtomicBool::new(false);

pub const USER_STACK_TOP: usize = 0x0000_7fff_ff00_0000;

pub fn early_system_init() {
    if SYSTEM_INIT_DONE
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    unsafe {
        idt::system_init();
        pic::remap_and_disable();

        early_cpu_init();

        let acpi_tables = acpi::init();
        let platform_info = ::acpi::PlatformInfo::new(&acpi_tables).unwrap();
        if let ::acpi::platform::interrupt::InterruptModel::Apic(apic) =
            platform_info.interrupt_model
        {
            ioapic::init(&apic);
        } else {
            panic!("no IOAPIC found");
        }

        ioapic::unmask_irq(4);

        // acpi_debug();

        let root = page::get_vm_root();
        println!("VM root: {:#x}", root as usize);
        page::init();
        // page::print_page_table(root);
    }
}

pub unsafe fn early_cpu_init() {
    cpu::use_();
    idt::load();
    lapic::init();
    lapic::start_timer();
}

pub fn direct_map_offset(phy: u64) -> usize {
    phy as usize + *DIRECT_MAP_OFFSET
}

pub fn direct_map<T>(ptr: *const T) -> *const T {
    (ptr as usize + *DIRECT_MAP_OFFSET) as *const T
}

pub fn direct_map_mut<T>(ptr: *mut T) -> *mut T {
    direct_map(ptr) as *mut T
}

pub fn interrupts_are_disabled() -> bool {
    unsafe {
        let flags: u64;
        asm!("pushfq", "pop {}", out(reg) flags);
        flags & 0x200 == 0
    }
}

#[allow(unused)]
pub fn send_ipi(cpu_id: u8, vector: u8) {
    lapic::send_ipi(cpu_id, vector);
}

pub fn broadcast_ipi(vector: u8) {
    lapic::broadcast_ipi(vector);
}

pub fn sleep_forever() -> ! {
    loop {
        unsafe { asm!("sti", "hlt") };
    }
}

pub fn sleep_forever_no_irq() -> ! {
    loop {
        unsafe { asm!("cli", "hlt") };
    }
}

#[allow(unused)]
pub fn pci_read(addr: PciAddress, offset: u8) -> u32 {
    let addr = addr.to_u32() | (offset as u32 & 0xfc);
    unsafe {
        pio::write_u32(0xcf8, 0x80000000 | addr);
        pio::read_u32(0xcfc)
    }
}

#[allow(unused)]
pub fn pci_write(addr: PciAddress, offset: u8, value: u32) {
    let addr = addr.to_u32() | (offset as u32 & 0xfc);
    unsafe {
        pio::write_u32(0xcf8, 0x80000000 | addr);
        pio::write_u32(0xcfc, value);
    }
}

#[allow(unused)]
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

pub fn print_backtrace() {
    let bp: usize;
    unsafe {
        asm!("mov {}, rbp", out(reg) bp);
    }
    print_backtrace_from(bp);
}

pub fn print_backtrace_from(mut bp: usize) {
    while bp != 0 {
        let ip = unsafe { *(bp as *const usize).offset(1) };
        println!("({:#x}) <>", ip);
        bp = unsafe { *(bp as *const usize) };

        if physical_address(bp).is_none() || physical_address(bp + 8).is_none() {
            println!("backtrace left mapped memory at {:x}", bp);
            break;
        }
    }
}

pub fn print_backtrace_from_with_ip(ip: usize, bp: usize) {
    println!("({:#x}) <>", ip);
    print_backtrace_from(bp);
}