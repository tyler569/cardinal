use crate::println;
use core::arch::asm;

#[derive(Copy, Clone, Debug)]
enum Ring {
    Ring0,
    Ring3,
}

impl Ring {
    fn as_u64(self) -> u64 {
        match self {
            Self::Ring0 => 0,
            Self::Ring3 => 3,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum GdtEntry {
    Empty,
    CodeSegment { ring: Ring },
    DataSegment { ring: Ring },
    TssSegment { address: u64, length: u32 },
}

#[allow(clippy::unusual_byte_groupings)]
impl GdtEntry {
    fn serialize(self) -> (u64, Option<u64>) {
        match self {
            Self::Empty => (0, None),
            Self::CodeSegment { ring } => (0x00_20_9a_00_0000_0000 | (ring.as_u64() << 45), None),
            Self::DataSegment { ring } => (0x00_20_92_00_0000_0000 | (ring.as_u64() << 45), None),
            Self::TssSegment { address, length } => {
                let low = address & 0xFFFF;
                let mid = (address >> 16) & 0xFF;
                let high = (address >> 24) & 0xFF;
                let extended = (address >> 32) & 0xFFFFFFFF;
                let limit = length as u64;
                assert!(limit < 0xFFFF);
                (
                    limit | (low << 16) | (mid << 32) | (0x89 << 40) | (high << 56),
                    Some(extended),
                )
            }
        }
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct GdtPtr {
    limit: u16,
    base: u64,
}

impl GdtPtr {
    fn new(gdt: &[u64]) -> Self {
        Self {
            limit: (core::mem::size_of_val(gdt) - 1) as u16,
            base: gdt.as_ptr() as u64,
        }
    }
}

fn basic_gdt(tss: &Tss) -> [GdtEntry; 6] {
    let tss_addr = tss as *const Tss as u64;
    [
        GdtEntry::Empty,
        GdtEntry::CodeSegment { ring: Ring::Ring0 },
        GdtEntry::DataSegment { ring: Ring::Ring0 },
        GdtEntry::CodeSegment { ring: Ring::Ring3 },
        GdtEntry::DataSegment { ring: Ring::Ring3 },
        GdtEntry::TssSegment {
            address: tss_addr,
            length: tss.len() as u32,
        },
    ]
}

fn serialize_gdt(entries: &[GdtEntry], gdt: &mut [u64]) {
    let mut i = 0;
    for entry in entries {
        let (low, high) = entry.serialize();
        gdt[i] = low;
        i += 1;
        if let Some(high) = high {
            gdt[i] = high;
            i += 1;
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Tss {
    reserved: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    reserved2: u64,
    ist1: u64,
    ist2: u64,
    ist3: u64,
    ist4: u64,
    ist5: u64,
    ist6: u64,
    ist7: u64,
    reserved3: u64,
    reserved4: u16,
    iomap_base: u16,
}

impl Tss {
    pub const fn new() -> Self {
        Self {
            reserved: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved2: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved3: 0,
            reserved4: 0,
            iomap_base: 0,
        }
    }

    fn len(&self) -> usize {
        core::mem::size_of::<Self>() - 1
    }

    pub fn kernel_stack(&self) -> u64 {
        self.rsp0
    }

    pub fn set_kernel_stack(&mut self, stack: u64) {
        self.rsp0 = stack;
    }

    pub fn set_df_stack(&mut self, stack: u64) {
        self.ist1 = stack;
    }
}

pub fn init_in_place(gdt: &mut [u64; 7], tss: &mut Tss) {
    let entries = basic_gdt(tss);
    serialize_gdt(&entries, gdt);
}

pub unsafe fn load(gdt: &[u64; 7]) {
    let ptr = GdtPtr::new(gdt);
    asm!("lgdt [{0}]", in(reg) &ptr);
    asm!("ltr ax", in("ax") 0x28);
}

#[allow(dead_code)]
pub fn debug_print(gdt: &[u64]) {
    for (i, entry) in gdt.iter().enumerate() {
        println!("gdt[{}]: {:#018x?}", i, entry);
    }
}
