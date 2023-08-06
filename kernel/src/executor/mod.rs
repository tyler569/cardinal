use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::AtomicBool;

struct Executor {
    work_to_do: AtomicBool,
}

impl Executor {
    fn new() -> Self {
        Self {
            work_to_do: AtomicBool::new(false),
        }
    }

    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {}
}
