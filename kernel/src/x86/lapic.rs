use crate::print::println;
use crate::x86;
use crate::x86::cpu;

pub const DEFAULT_ADDRESS: usize = 0xfee0_0000;
pub const MAPPED_ADDRESS: *mut u32 = (DEFAULT_ADDRESS + 0xFFFF_8000_0000_0000) as *mut u32;

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
    let ptr = MAPPED_ADDRESS.byte_offset(offset);
    ptr.read_volatile()
}

pub unsafe fn write(offset: isize, value: u32) {
    let ptr = MAPPED_ADDRESS.byte_offset(offset);
    ptr.write_volatile(value);
}

pub fn eoi() {
    unsafe {
        write(0xb0, 0);
    }
}

pub fn send_ipi(apic_id: u8, vector: u8) {
    unsafe {
        write(0x310, (apic_id as u32) << 24);
        write(0x300, vector as u32 | 1 << 14);
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
