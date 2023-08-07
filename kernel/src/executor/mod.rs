use crate::per_cpu::PerCpu;
use crate::print::{print, println};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::AtomicBool;
use core::task::{Context, Poll, RawWaker, RawWakerVTable};
use spin::Mutex;

pub mod sleep;

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

pub struct Executor {
    work_to_do: AtomicBool,
    tasks_to_poll: Mutex<VecDeque<usize>>,
    next_id: usize,
    tasks: BTreeMap<usize, Task>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            work_to_do: AtomicBool::new(false),
            tasks_to_poll: Mutex::new(VecDeque::new()),
            next_id: 0,
            tasks: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let id = self.next_id;
        self.next_id += 1;
        let task = Task {
            future: Box::pin(future),
        };
        self.tasks.insert(id, task);
        self.tasks_to_poll.lock().push_back(id);
    }

    pub fn do_work(&mut self) {
        let mut tasks_to_poll = self.tasks_to_poll.lock();
        while let Some(id) = tasks_to_poll.pop_front() {
            let task = self.tasks.get_mut(&id).unwrap();
            let waker = new_waker(id);
            let mut context = Context::from_waker(&waker);
            if let Poll::Ready(()) = task.future.as_mut().poll(&mut context) {
                self.tasks.remove(&id);
            }
        }
    }
}

const WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(exec_clone, exec_wake, exec_wake_by_ref, exec_drop);

fn exec_clone(data: *const ()) -> RawWaker {
    // println!("[async] exec_clone");
    RawWaker::new(data, &WAKER_VTABLE)
}

unsafe fn exec_wake(data: *const ()) {
    // println!("[async] exec_wake");
    let id = data as usize;
    let executor = &PerCpu::get().executor;
    executor.tasks_to_poll.lock().push_back(id);
}

unsafe fn exec_wake_by_ref(data: *const ()) {
    // println!("[async] exec_wake_by_ref");
    exec_wake(data)
}

unsafe fn exec_drop(data: *const ()) {
    // println!("[async] exec_drop");
    // let _executor = Box::from_raw(data as *mut Executor);
}

fn new_waker(id: usize) -> core::task::Waker {
    let raw_waker = RawWaker::new(id as *const (), &WAKER_VTABLE);
    unsafe { core::task::Waker::from_raw(raw_waker) }
}
