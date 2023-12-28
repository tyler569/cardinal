use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Executor {
    work_to_do: AtomicBool,
    tasks_to_poll: VecDeque<u64>,
    next_id: AtomicU64,
    tasks: BTreeMap<u64, Task>,
}

static mut EXECUTOR: Executor = Executor::new();

impl Executor {
    pub const fn new() -> Self {
        Self {
            work_to_do: AtomicBool::new(false),
            tasks_to_poll: VecDeque::new(),
            next_id: AtomicU64::new(1),
            tasks: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, _future: impl Future<Output = ()> + 'static) {
        let task = Task {
            future: Box::pin(_future),
        };
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.tasks.insert(id, task);
    }

    pub fn do_work(&mut self) {
        while let Some(id) = self.tasks_to_poll.pop_front() {
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
    id: u64,
}

const WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(exec_clone, exec_wake, exec_wake_by_ref, exec_drop);

fn exec_clone(data: *const ()) -> RawWaker {
    let data = Box::into_raw(Box::new(unsafe { *(data as *mut WakerData) }));
    RawWaker::new(data as *const (), &WAKER_VTABLE)
}

unsafe fn exec_wake(data: *const ()) {
    let wd = *(data as *mut WakerData);
    unsafe {
        EXECUTOR.tasks_to_poll.push_back(wd.id);
        EXECUTOR.work_to_do.store(true, Ordering::SeqCst);
    };
}

unsafe fn exec_wake_by_ref(data: *const ()) {
    exec_wake(data)
}

unsafe fn exec_drop(data: *const ()) {
    let _ = Arc::from_raw(data as *mut WakerData);
}

fn new_waker(id: u64) -> Waker {
    let data = Arc::new(WakerData {
        id,
    });
    let data = Arc::into_raw(data);
    let raw_waker = RawWaker::new(data as *const (), &WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}
