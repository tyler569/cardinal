use crate::executor::Executor;
use crate::timer::Timer;
use crate::{arch, NUM_CPUS};
use core::cell::UnsafeCell;
use core::ops::{Index, IndexMut};
use spin::Lazy;

pub struct PerCpu {
    this: *const UnsafeCell<Self>,
    arch: arch::Cpu,
    timer: Timer,
    executor: Executor,
    running: Option<u64>,
}

impl PerCpu {
    fn new() -> Self {
        Self {
            this: core::ptr::null(),
            arch: arch::Cpu::new(),
            timer: Timer::new(),
            executor: Executor::new(),
            running: None,
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

    pub fn executor_for_cpu(cpu: usize) -> &'static Executor {
        &unsafe { &*PER_CPU[cpu].get() }.executor
    }

    pub fn running() -> Option<u64> {
        Self::get().running
    }

    pub fn set_running(proc: Option<u64>) {
        Self::get_mut().running = proc;
    }

    pub fn ticks() -> u64 {
        Self::get().timer.ticks()
    }

    pub fn executor_mut() -> &'static mut Executor {
        &mut Self::get_mut().executor
    }

    pub fn timer_mut() -> &'static mut Timer {
        &mut Self::get_mut().timer
    }

    pub fn arch() -> &'static arch::Cpu {
        &Self::get().arch
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

static PER_CPU: Lazy<PerCpuContainer> = Lazy::new(PerCpuContainer::new);
