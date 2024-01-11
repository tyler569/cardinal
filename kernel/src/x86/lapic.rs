use spin::Lazy;
use crate::print::println;
use crate::x86::{cpu, direct_map_offset};

pub const DEFAULT_ADDRESS: u64 = 0xfee0_0000;
pub static MAPPED_ADDRESS: Lazy<usize> = Lazy::new(|| {
    direct_map_offset(DEFAULT_ADDRESS)
});

unsafe fn relocate() {
    cpu::wrmsr(cpu::IA32_LAPIC_BASE, DEFAULT_ADDRESS as u64 | 1 << 11);
}

pub fn init() {
    unsafe {
        relocate();
        write(0xf0, read(0xf0) | 0x100);
    }
}

pub unsafe fn read(offset: isize) -> u32 {
    let ptr = (MAPPED_ADDRESS.clone() as *mut u32).byte_offset(offset);
    ptr.read_volatile()
}

pub unsafe fn write(offset: isize, value: u32) {
    let ptr = (MAPPED_ADDRESS.clone() as *mut u32).byte_offset(offset);
    ptr.write_volatile(value);
}

pub fn eoi() {
    unsafe {
        write(0xb0, 0);
    }
}

#[allow(unused)]
pub fn send_ipi(apic_id: u8, vector: u8) {
    unsafe {
        write(0x310, (apic_id as u32) << 24);
        write(0x300, vector as u32 | 1 << 14);
    }
}

pub fn broadcast_ipi(vector: u8) {
    unsafe {
        // send ipi to all other CPUS except self
        write(0x310, 0);
        write(0x300, vector as u32 | 1 << 14 | 3 << 18);
    }
}

pub fn start_timer() {
    println!("starting timer");
    unsafe {
        write(0x320, 1 << 17 | 0x20);
        write(0x3e0, 0b1011);
        write(0x380, 1_000_000);
    }
}
