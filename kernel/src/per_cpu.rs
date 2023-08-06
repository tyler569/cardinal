use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::sync::atomic::AtomicU64;
use spin::Lazy;
use crate::{arch, NUM_CPUS};

pub struct PerCpu {
    this: *const UnsafeCell<Self>,
    pub arch: arch::Cpu,
    pub ticks: AtomicU64,
}

impl PerCpu {
    const fn new() -> Self {
        Self {
            this: 0 as *const _,
            ticks: AtomicU64::new(0),
            arch: arch::Cpu::new(),
        }
    }

    pub fn init() {
        for (i, cell) in PER_CPU.cpus.iter().enumerate() {
            let this = unsafe { &mut *cell.get() };
            this.this = cell;
            this.arch.setup(i);
        }
    }

    pub fn get() -> &'static Self {
        unsafe { &*PER_CPU[arch::cpu_num() as usize].get() }
    }

    pub fn get_mut() -> &'static mut Self {
        unsafe { &mut *PER_CPU[arch::cpu_num() as usize].get() }
    }
}

// This is all hidden and only exists to impl `Sync` on the UnsafeCell in PerCpu

struct PerCpuContainer {
    cpus: [UnsafeCell<PerCpu>; NUM_CPUS],
}

impl PerCpuContainer {
    fn new() -> Self {
        Self {
            cpus: array_init::array_init(|_| UnsafeCell::new(PerCpu::new())),
        }
    }
}

impl Index<usize> for PerCpuContainer {
    type Output = UnsafeCell<PerCpu>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cpus[index]
    }
}

impl IndexMut<usize> for PerCpuContainer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cpus[index]
    }
}

// Safety: each CPU has its own PerCpu instance, and each PerCpu instance is
// only accessed by the CPU it belongs to.
unsafe impl Send for PerCpuContainer {}
unsafe impl Sync for PerCpuContainer {}

static PER_CPU: Lazy<PerCpuContainer> = Lazy::new(|| PerCpuContainer::new());