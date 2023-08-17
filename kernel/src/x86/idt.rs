use crate::print::println;
use crate::x86::frame::InterruptFrame;
use core::arch::asm;

static mut IDT: Idt = Idt::zero();

#[repr(align(8))]
struct Idt {
    table: [IdtEntry; 256],
}

impl Idt {
    pub const fn zero() -> Self {
        Self {
            table: [IdtEntry::zero(); 256],
        }
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }
}

impl core::ops::Index<usize> for Idt {
    type Output = IdtEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

impl core::ops::IndexMut<usize> for Idt {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.table[index]
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    offset_middle: u16,
    offset_high: u32,
    zero: u32,
}

pub enum GateType {
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

impl IdtEntry {
    pub const fn zero() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_middle: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn new(handler: unsafe extern "C" fn(), ist: u8, rpl: u8, typ: GateType) -> Self {
        let offset = handler as u64;
        let offset_low = offset as u16;
        let offset_middle = (offset >> 16) as u16;
        let offset_high = (offset >> 32) as u32;

        let flags = 0x80 | (rpl << 5) | typ as u8;

        Self {
            offset_low,
            selector: 8, // kernel CS
            ist,
            flags,
            offset_middle,
            offset_high,
            zero: 0,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct IdtPtr {
    size: u16,
    base: u64,
}

pub(super) unsafe fn system_init() {
    IDT[0] = IdtEntry::new(isr0, 0, 0, GateType::InterruptGate);
    IDT[1] = IdtEntry::new(isr1, 0, 0, GateType::InterruptGate);
    IDT[2] = IdtEntry::new(isr2, 0, 0, GateType::InterruptGate);
    IDT[3] = IdtEntry::new(isr3, 0, 3, GateType::InterruptGate);
    IDT[4] = IdtEntry::new(isr4, 0, 0, GateType::InterruptGate);
    IDT[5] = IdtEntry::new(isr5, 0, 0, GateType::InterruptGate);
    IDT[6] = IdtEntry::new(isr6, 0, 0, GateType::InterruptGate);
    IDT[7] = IdtEntry::new(isr7, 0, 0, GateType::InterruptGate);
    IDT[8] = IdtEntry::new(isr8, 1, 0, GateType::InterruptGate);
    IDT[9] = IdtEntry::new(isr9, 0, 0, GateType::InterruptGate);
    IDT[10] = IdtEntry::new(isr10, 0, 0, GateType::InterruptGate);
    IDT[11] = IdtEntry::new(isr11, 0, 0, GateType::InterruptGate);
    IDT[12] = IdtEntry::new(isr12, 0, 0, GateType::InterruptGate);
    IDT[13] = IdtEntry::new(isr13, 0, 0, GateType::InterruptGate);
    IDT[14] = IdtEntry::new(isr14, 0, 0, GateType::InterruptGate);
    IDT[15] = IdtEntry::new(isr15, 0, 0, GateType::InterruptGate);
    IDT[16] = IdtEntry::new(isr16, 0, 0, GateType::InterruptGate);
    IDT[17] = IdtEntry::new(isr17, 0, 0, GateType::InterruptGate);
    IDT[18] = IdtEntry::new(isr18, 0, 0, GateType::InterruptGate);
    IDT[19] = IdtEntry::new(isr19, 0, 0, GateType::InterruptGate);
    IDT[20] = IdtEntry::new(isr20, 0, 0, GateType::InterruptGate);
    IDT[21] = IdtEntry::new(isr21, 0, 0, GateType::InterruptGate);
    IDT[22] = IdtEntry::new(isr22, 0, 0, GateType::InterruptGate);
    IDT[23] = IdtEntry::new(isr23, 0, 0, GateType::InterruptGate);
    IDT[24] = IdtEntry::new(isr24, 0, 0, GateType::InterruptGate);
    IDT[25] = IdtEntry::new(isr25, 0, 0, GateType::InterruptGate);
    IDT[26] = IdtEntry::new(isr26, 0, 0, GateType::InterruptGate);
    IDT[27] = IdtEntry::new(isr27, 0, 0, GateType::InterruptGate);
    IDT[28] = IdtEntry::new(isr28, 0, 0, GateType::InterruptGate);
    IDT[29] = IdtEntry::new(isr29, 0, 0, GateType::InterruptGate);
    IDT[30] = IdtEntry::new(isr30, 0, 0, GateType::InterruptGate);
    IDT[31] = IdtEntry::new(isr31, 0, 0, GateType::InterruptGate);
    IDT[32] = IdtEntry::new(irq0, 0, 0, GateType::InterruptGate);
    IDT[33] = IdtEntry::new(irq1, 0, 0, GateType::InterruptGate);
    IDT[34] = IdtEntry::new(irq2, 0, 0, GateType::InterruptGate);
    IDT[35] = IdtEntry::new(irq3, 0, 0, GateType::InterruptGate);
    IDT[36] = IdtEntry::new(irq4, 0, 0, GateType::InterruptGate);
    IDT[37] = IdtEntry::new(irq5, 0, 0, GateType::InterruptGate);
    IDT[38] = IdtEntry::new(irq6, 0, 0, GateType::InterruptGate);
    IDT[39] = IdtEntry::new(irq7, 0, 0, GateType::InterruptGate);
    IDT[40] = IdtEntry::new(irq8, 0, 0, GateType::InterruptGate);
    IDT[41] = IdtEntry::new(irq9, 0, 0, GateType::InterruptGate);
    IDT[42] = IdtEntry::new(irq10, 0, 0, GateType::InterruptGate);
    IDT[43] = IdtEntry::new(irq11, 0, 0, GateType::InterruptGate);
    IDT[44] = IdtEntry::new(irq12, 0, 0, GateType::InterruptGate);
    IDT[45] = IdtEntry::new(irq13, 0, 0, GateType::InterruptGate);
    IDT[46] = IdtEntry::new(irq14, 0, 0, GateType::InterruptGate);
    IDT[47] = IdtEntry::new(irq15, 0, 0, GateType::InterruptGate);
    IDT[128] = IdtEntry::new(isr_syscall, 0, 3, GateType::InterruptGate);
    IDT[129] = IdtEntry::new(isr_ipi, 0, 0, GateType::InterruptGate);
    IDT[130] = IdtEntry::new(isr_ipi_panic, 0, 0, GateType::InterruptGate);
}

pub(super) unsafe fn load() {
    let ptr = IdtPtr {
        size: (core::mem::size_of::<IdtEntry>() * IDT.len() - 1) as u16,
        base: &IDT as *const _ as u64,
    };

    asm!("lidt [{0}]", in(reg) &ptr);
}

#[naked]
#[no_mangle]
unsafe extern "C" fn interrupt_shim() {
    asm!(
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov ebp, ds",
        "push rbp",
        "xor rbp, rbp",
        "mov ds, ebp",
        "mov rdi, rsp",

        "and rsp, 0xfffffffffffffff0",
        "push rsp",
        "push 0",

        "call rs_interrupt_shim",

        "add rsp, 8",
        "pop rsp",
        "add rsp, 8",

        "pop rbp",
        "mov ds, ebp",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "add rsp, 16",
        "iretq",
        options(noreturn),
    );
}

macro_rules! isr_no_error {
    ($name:ident, $num:expr) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $name() {
            asm!(
                "push 0",
                concat!("push ", stringify!($num)),
                "jmp interrupt_shim",
                options(noreturn),
            )
        }
    };
}

macro_rules! isr_error {
    ($name:ident, $num:expr) => {
        #[naked]
        #[no_mangle]
        unsafe extern "C" fn $name() {
            asm!(
                concat!("push ", stringify!($num)),
                "jmp interrupt_shim",
                options(noreturn),
            )
        }
    };
}

isr_no_error!(isr0, 0);
isr_no_error!(isr1, 1);
isr_no_error!(isr2, 2);
isr_no_error!(isr3, 3);
isr_no_error!(isr4, 4);
isr_no_error!(isr5, 5);
isr_no_error!(isr6, 6);
isr_no_error!(isr7, 7);
isr_error!(isr8, 8);
isr_no_error!(isr9, 9);
isr_error!(isr10, 10);
isr_error!(isr11, 11);
isr_error!(isr12, 12);
isr_error!(isr13, 13);
isr_error!(isr14, 14);
isr_no_error!(isr15, 15);
isr_no_error!(isr16, 16);
isr_error!(isr17, 17);
isr_no_error!(isr18, 18);
isr_no_error!(isr19, 19);
isr_no_error!(isr20, 20);
isr_no_error!(isr21, 21);
isr_no_error!(isr22, 22);
isr_no_error!(isr23, 23);
isr_no_error!(isr24, 24);
isr_no_error!(isr25, 25);
isr_no_error!(isr26, 26);
isr_no_error!(isr27, 27);
isr_no_error!(isr28, 28);
isr_no_error!(isr29, 29);
isr_no_error!(isr30, 30);
isr_no_error!(isr31, 31);
isr_no_error!(irq0, 32);
isr_no_error!(irq1, 33);
isr_no_error!(irq2, 34);
isr_no_error!(irq3, 35);
isr_no_error!(irq4, 36);
isr_no_error!(irq5, 37);
isr_no_error!(irq6, 38);
isr_no_error!(irq7, 39);
isr_no_error!(irq8, 40);
isr_no_error!(irq9, 41);
isr_no_error!(irq10, 42);
isr_no_error!(irq11, 43);
isr_no_error!(irq12, 44);
isr_no_error!(irq13, 45);
isr_no_error!(irq14, 46);
isr_no_error!(irq15, 47);
isr_no_error!(isr_syscall, 128);
isr_no_error!(isr_ipi, 129);
isr_no_error!(isr_ipi_panic, 130);
