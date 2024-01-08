use crate::per_cpu::PerCpu;
use crate::x86::gdt;
use crate::x86::gdt::Tss;
use crate::NUM_CPUS;
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

#[allow(dead_code)]
pub unsafe fn wrmsr(msr: u32, value: u64) {
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") value,
        in("edx") value >> 32,
    );
}

#[allow(dead_code)]
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

pub fn cr2() -> u64 {
    let value: u64;
    unsafe {
        asm!(
            "mov {0:r}, cr2",
            out(reg) value,
        );
    }
    value
}

pub fn cpu_num() -> usize {
    (cpuid(1, 0)[1] >> 24) as usize
}

#[derive(Copy, Clone)]
pub struct Cpu {
    initialized: bool,
    gdt: [u64; 7],
    tss: Tss,
    stack: *const Stack,
    df_stack: *const Stack,
}

impl Cpu {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            gdt: [0; 7],
            tss: Tss::new(),
            stack: core::ptr::null(),
            df_stack: core::ptr::null(),
        }
    }

    pub fn setup(&mut self, cpu_num: usize) {
        if self.initialized {
            panic!("CPU already initialized");
        }

        self.initialized = true;

        gdt::init_in_place(&mut self.gdt, &mut self.tss);

        self.stack = unsafe { &STACKS[cpu_num].0 } as *const _;
        self.df_stack = unsafe { &STACKS[cpu_num].1 } as *const _;

        self.tss
            .set_kernel_stack(unsafe { (*self.stack).top() as u64 });
        self.tss
            .set_df_stack(unsafe { (*self.df_stack).top() as u64 });
    }

    fn use_(&self) {
        unsafe { gdt::load(&self.gdt) };
    }
}

static mut STACKS: [(Stack, Stack); NUM_CPUS] = [(Stack::new(), Stack::new()); NUM_CPUS];

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

pub unsafe fn use_() {
    let cpu = PerCpu::arch();
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

pub fn kernel_stack() -> u64 {
    PerCpu::arch().tss.kernel_stack()
}
