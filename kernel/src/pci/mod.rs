use core::fmt::{Debug, Display};
use crate::arch;
use crate::pci::rtl8139::Rtl8139;
use crate::print::println;

mod rtl8139;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PciAddress {
    bus: u8,
    device: u8,
    function: u8,
}

impl PciAddress {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }

    pub fn to_u32(&self) -> u32 {
        ((self.bus as u32) << 16) | ((self.device as u32) << 11) | ((self.function as u32) << 8)
    }
}

impl Debug for PciAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PciAddress({})", self)
    }
}

impl Display for PciAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}:{:02x}.{}", self.bus, self.device, self.function)
    }
}

pub fn enumerate_pci_bus() {
    for bus in 0..255 {
        for slot in 0..255 {
            for function in 0..8 {
                print_device_info(PciAddress::new(bus, slot, function));
            }
        }
    }
}

fn print_device_info(address: PciAddress) {
    let base = arch::pci_read(address, 0);
    if base == 0xffff_ffff {
        return;
    }

    let vendor_id = base & 0xffff;
    let device_id = (base >> 16) & 0xffff;
    let class_code = arch::pci_read(address, 8);
    let subclass = (class_code >> 16) & 0xff;
    let class = (class_code >> 24) & 0xff;
    let prog_if = (class_code >> 8) & 0xff;
    let revision = (class_code >> 0) & 0xff;
    let header_type = (arch::pci_read(address, 12) >> 16) & 0xff;

    println!("PCI device {}: {:04x}:{:04x} (class {:02x}:{:02x}:{:02x} rev {:02x})",
        address, vendor_id, device_id,
        class, subclass, prog_if, revision,
    );

    if vendor_id as u16 == Rtl8139::VENDOR_ID && device_id as u16 == Rtl8139::DEVICE_ID {
        let rtl8139 = Rtl8139::new(address);
        println!("  RTL8139: {:?}", rtl8139);
    }
}