use crate::print::println;
use crate::x86::gdt;
use crate::x86::gdt::Tss;
use core::arch::asm;

pub fn cpuid(a: u32, c: u32) -> [u32; 4] {
    let mut result: [u32; 4] = [0; 4];
    unsafe {
        asm!(
            "xchg {0:r}, rbx",
            "cpuid",
            "xchg {0:r}, rbx",
            lateout(reg) result[1],
            in("eax") a,
            in("ecx") c,
            lateout("eax") result[0],
            lateout("ecx") result[2],
            lateout("edx") result[3],
        )
    }
    result
}

pub const IA32_LAPIC_BASE: u32 = 27;

pub unsafe fn wrmsr(msr: u32, value: u64) {
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") value,
        in("edx") value >> 32,
    );
}

pub unsafe fn rdmsr(msr: u32) -> u64 {
    let value_low: u64;
    let value_high: u64;
    asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") value_low,
        out("edx") value_high
    );
    (value_high << 32) + value_low
}

pub fn cpu_num() -> u32 {
    cpuid(1, 0)[1] >> 24
}

#[derive(Copy, Clone)]
struct Cpu {
    initialized: bool,
    gdt: [u64; 7],
    tss: Tss,
    stack: Stack,
    df_stack: Stack,
}

impl Cpu {
    const fn new() -> Self {
        Self {
            initialized: false,
            gdt: [0; 7],
            tss: Tss::new(),
            stack: Stack::new(),
            df_stack: Stack::new(),
        }
    }

    fn setup(&mut self) {
        if self.initialized {
            panic!("CPU already initialized");
        }

        self.initialized = true;

        unsafe { gdt::init_in_place(&mut self.gdt, &mut self.tss) };

        self.tss.set_kernel_stack(self.stack.top() as u64);
        self.tss.set_df_stack(self.df_stack.top() as u64);
    }

    fn use_(&self) {
        unsafe { gdt::load(&self.gdt) };
    }
}

static mut CPUS: [Cpu; 16] = [Cpu::new(); 16];

#[repr(align(16))]
#[derive(Copy, Clone)]
struct Stack([u8; Self::SIZE]);

impl Stack {
    pub const SIZE: usize = 4096 * 4;

    const fn new() -> Self {
        Self([0u8; Self::SIZE])
    }

    pub fn top(&self) -> usize {
        self.0.as_ptr() as usize + Self::SIZE
    }
}

pub(crate) unsafe fn system_init() {
    for cpu in CPUS.iter_mut() {
        cpu.setup();
    }
}

pub(crate) unsafe fn use_() {
    let cpu = &CPUS[cpu_num() as usize];
    cpu.use_();
    asm!(
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
        in("ax") 0,
    );
}

pub(crate) fn kernel_stack() -> u64 {
    unsafe { CPUS[cpu_num() as usize] }.tss.kernel_stack()
}