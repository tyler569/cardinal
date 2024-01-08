use alloc::boxed::Box;
use crate::NUM_CPUS;
use crate::per_cpu::PerCpu;
use crate::arch::broadcast_ipi;

pub struct IpiFunction {
    function: Box<dyn FnOnce()>,
}

impl IpiFunction {
    pub fn new<F: FnOnce() + 'static>(function: F) -> Self {
        Self {
            function: Box::new(function)
        }
    }

    pub fn call(self) {
        (self.function)()
    }
}

pub fn handle_ipi_irq() {
    PerCpu::ipi_queue().lock().drain(..).for_each(|func| func.call());
}

pub fn submit_ipi_to_all_cpus<F: FnOnce() + Clone + 'static>(function: F) {
    for i in 0..NUM_CPUS {
        PerCpu::submit_ipi(i, function.clone())
    }

    broadcast_ipi(129);
}