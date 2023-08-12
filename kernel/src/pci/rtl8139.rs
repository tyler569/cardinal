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

    tx_slot: usize,
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

        let mut res = Self {
            address,
            io_base: io_base as usize,
            io_size: io_size as usize,
            irq,
            mac: MacAddress::new( [0; 6]),
            tx_slot: 0,
        };
        let mut mac = [0; 6];
        for i in 0..6 {
            mac[i] = unsafe { res.io_read_u8(i) };
        }
        res.mac = MacAddress::new(mac);
        res
    }

    fn io_ptr(&self, offset: usize) -> *mut u8 {
        arch::direct_map_mut((self.io_base + offset) as *mut u8)
    }

    unsafe fn io_read_u8(&self, offset: usize) -> u8 {
        (self.io_ptr(offset) as *const u8).read_volatile()
    }

    unsafe fn io_read_u16(&self, offset: usize) -> u16 {
        (self.io_ptr(offset) as *const u16).read_volatile()
    }

    unsafe fn io_read_u32(&self, offset: usize) -> u32 {
        (self.io_ptr(offset) as *const u32).read_volatile()
    }

    unsafe fn io_write_u8(&self, offset: usize, value: u8) {
        (self.io_ptr(offset)).write_volatile(value)
    }

    unsafe fn io_write_u16(&self, offset: usize, value: u16) {
        (self.io_ptr(offset) as *mut u16).write_volatile(value)
    }

    unsafe fn io_write_u32(&self, offset: usize, value: u32) {
        (self.io_ptr(offset) as *mut u32).write_volatile(value)
    }


    pub fn init(&mut self) {
        self.reset();
    }

    pub fn reset(&mut self) {
        unsafe {
            // enable bus mastering
            let bus_state = arch::pci_read(self.address, 0x4);
            arch::pci_write(self.address, 0x4, bus_state | 0x4);

            self.io_write_u8(0x52, 0x0); // power on
            self.io_write_u8(0x37, 0x10); // reset
            while self.io_read_u8(0x37) & 0x10 != 0 {} // wait for reset

            println!("RTL8139 reset");

            let ring_phy = crate::pmm::alloc_contiguous(16).unwrap();
            let ring_mapped = arch::direct_map_offset(ring_phy);

            self.io_write_u32(0x30, ring_phy as u32); // ring buffer
            self.io_write_u16(0x3c, 0x0005); // configure interrupts and txok, rxok

            self.io_write_u32(0x40, 0x600); // send larger DMA bursts
            self.io_write_u32(0x44, 0x68f); // accept all packets + unlimited DMA

            self.io_write_u8(0x37, 0x0c); // enable rx and tx
            self.tx_slot = 0;
        }
    }

    pub fn send_packet(&mut self, data: &[u8]) {
        unsafe {
            if data.len() > 1500 {
                panic!("Tried to send oversize packet on rtl8139");
            }

            let Some(phy_data) = arch::physical_address(data.as_ptr() as usize) else {
                println!("rtl8139 attempted to send packet from unmapped region");
                return;
            };
            if phy_data > 0xffff_ffff {
                println!("rtl8139 can't send packets from above physical 4G");
                return;
            }
            let tx_addr_off = 0x20 + self.tx_slot * 4;
            let ctrl_reg_off = 0x10 + self.tx_slot * 4;

            self.io_write_u32(tx_addr_off, phy_data as u32);
            self.io_write_u32(ctrl_reg_off, data.len() as u32);

            // TODO: async this and check when we get back around to the slot

            // await device taking packet
            while self.io_read_u32(ctrl_reg_off) & 0x100 != 0 {}
            // await send confirmation
            while self.io_read_u32(ctrl_reg_off) & 0x400 != 0 {}

            self.tx_slot += 1;
            self.tx_slot %= 4;
        }

    }
}