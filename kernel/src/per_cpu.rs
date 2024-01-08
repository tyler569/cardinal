use alloc::collections::VecDeque;
use crate::executor::Executor;
use crate::timer::Timer;
use crate::{arch, NUM_CPUS};
use core::cell::UnsafeCell;
use core::ops::{Index, IndexMut};
use spin::{Lazy, Mutex};
use crate::ipi::IpiFunction;
use crate::x86::cpu_num;

pub struct PerCpu {
    this: *const UnsafeCell<Self>,
    arch: arch::Cpu,
    timer: Timer,
    executor: Executor,
    running: Option<u64>,
    ipi_queue: Mutex<VecDeque<IpiFunction>>
}

impl PerCpu {
    fn new() -> Self {
        Self {
            this: core::ptr::null(),
            arch: arch::Cpu::new(),
            timer: Timer::new(),
            executor: Executor::new(),
            running: None,
            ipi_queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn init() {
        for (i, cell) in PER_CPU.cpus.iter().enumerate() {
            let this = unsafe { &mut *cell.get() };
            this.this = cell;
            this.arch.setup(i);
        }
    }

    pub unsafe fn cpu(cpu: usize) -> &'static Self {
        &*PER_CPU[cpu].get()
    }

    // unsafe fn cpu_mut is impossible because we maintain an invariant that mutable access to PerCPU
    // is only performed by the current CPU.

    pub fn get() -> &'static Self {
        unsafe { Self::cpu(cpu_num()) }
    }

    pub fn get_mut() -> &'static mut Self {
        unsafe { &mut *PER_CPU[cpu_num()].get() }
    }

    pub fn executor_for_cpu(cpu: usize) -> &'static Executor {
        &unsafe { Self::cpu(cpu) }.executor
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

    pub fn submit_ipi<F: FnOnce() + 'static>(cpu: usize, function: F) {
        unsafe { Self::cpu(cpu) }.ipi_queue.lock().push_back(IpiFunction::new(function));
    }

    pub fn ipi_queue() -> &'static Mutex<VecDeque<IpiFunction>> {
        &Self::get().ipi_queue
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
