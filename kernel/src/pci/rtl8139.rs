use crate::arch;
use crate::net::MacAddress;
use crate::pci::PciAddress;
use crate::print::println;

#[derive(Debug)]
pub struct Rtl8139 {
    address: PciAddress,
    io_base: usize,
    io_size: usize,
    irq: u8,
    mac: MacAddress,
}

impl Rtl8139 {
    pub const VENDOR_ID: u16 = 0x10ec;
    pub const DEVICE_ID: u16 = 0x8139;

    pub fn new(address: PciAddress) -> Self {
        let mut use_bar = None;
        for bar in 0..6 {
            let base = arch::pci_read(address, (0x10 + bar * 4) as u8);
            if base & 1 == 0 {
                use_bar = Some(bar);
                break;
            }
        }

        let Some(use_bar) = use_bar else {
            panic!("No usable BAR found for RTL8139");
        };
        let bar_offset = (0x10 + use_bar * 4) as u8;

        let io_base = arch::pci_read(address, bar_offset);
        arch::pci_write(address, bar_offset, 0xffff_ffff);
        let io_size = !arch::pci_read(address, bar_offset) + 1;
        arch::pci_write(address, bar_offset, io_base);

        let irq = arch::pci_read(address, 0x3c) as u8 & 0xf;
        let mut mac = [0; 6];
        for i in 0..6 {
            mac[i] = unsafe { (io_base as *const u8).add(i).read_volatile() }
        }

        Self {
            address,
            io_base: io_base as usize,
            io_size: io_size as usize,
            irq,
            mac: MacAddress::new(mac),
        }
    }

    pub fn io_read_u32(&self, offset: usize) -> u32 {
        unsafe { ((self.io_base + offset) as *const u32).read_volatile() }
    }

    pub fn io_write_u32(&self, offset: usize, value: u32) {
        unsafe { ((self.io_base + offset) as *mut u32).write_volatile(value) }
    }

    pub fn io_read_u8(&self, offset: usize) -> u8 {
        unsafe { ((self.io_base + offset) as *const u8).read_volatile() }
    }

    pub fn io_write_u8(&self, offset: usize, value: u8) {
        unsafe { ((self.io_base + offset) as *mut u8).write_volatile(value) }
    }

    pub fn init(&mut self) {
        self.reset();
    }

    pub fn reset(&mut self) {
        // enable bus mastering
        let bus_state = arch::pci_read(self.address, 0x4);
        arch::pci_write(self.address, 0x4, bus_state | 0x4);

        self.io_write_u8(0x52, 0x0); // power on
        self.io_write_u8(0x37, 0x10); // reset
        while self.io_read_u8(0x37) & 0x10 != 0 {} // wait for reset

        println!("RTL8139 reset");
    }
}