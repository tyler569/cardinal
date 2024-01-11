use crate::arch;
use crate::per_cpu::PerCpu;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use spin::Mutex;

pub mod sleep;

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Executor {
    tasks_to_poll: Mutex<VecDeque<u64>>,
    next_id: AtomicU64,
    tasks: BTreeMap<u64, Task>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks_to_poll: Mutex::new(VecDeque::new()),
            next_id: AtomicU64::new(1),
            tasks: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        assert!(arch::interrupts_are_disabled());
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let task = Task {
            future: Box::pin(future),
        };
        self.tasks.insert(id, task);
        self.tasks_to_poll.lock().push_back(id);
    }

    pub fn do_work(&mut self) {
        assert!(arch::interrupts_are_disabled());
        loop {
            let Some(id) = self.tasks_to_poll.lock().pop_front() else {
                break;
            };
            let task = self.tasks.get_mut(&id).unwrap();
            let waker = new_waker(id);
            let mut context = Context::from_waker(&waker);
            if let Poll::Ready(()) = task.future.as_mut().poll(&mut context) {
                self.tasks.remove(&id);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct WakerData {
    cpu: usize,
    id: u64,
}

const WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(exec_clone, exec_wake, exec_wake_by_ref, exec_drop);

fn exec_clone(data: *const ()) -> RawWaker {
    let new_ref = unsafe { Arc::from_raw(data as *const WakerData) };
    let data = Arc::into_raw(new_ref.clone());
    let _ = Arc::into_raw(new_ref);
    RawWaker::new(data as *const (), &WAKER_VTABLE)
}

unsafe fn exec_wake(data: *const ()) {
    assert!(arch::interrupts_are_disabled());
    let wd = *(data as *mut WakerData);
    let executor = PerCpu::executor_for_cpu(wd.cpu as usize);
    executor.tasks_to_poll.lock().push_back(wd.id);
}

unsafe fn exec_wake_by_ref(data: *const ()) {
    exec_wake(data)
}

unsafe fn exec_drop(data: *const ()) {
    let _ = Arc::from_raw(data as *mut WakerData);
}

fn new_waker(id: u64) -> Waker {
    let data = Arc::new(WakerData {
        cpu: arch::cpu_num(),
        id,
    });
    let data = Arc::into_raw(data);
    let raw_waker = RawWaker::new(data as *const (), &WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

pub fn spawn(future: impl Future<Output = ()> + 'static) {
    PerCpu::executor_mut().spawn(future)
}
