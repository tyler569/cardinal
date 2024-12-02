use crate::syscall::syscall_future;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use cardinal3_interface::{Syscall, SyscallReturn, SyscallContext};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

pub struct Executor {
    tasks_to_poll: VecDeque<u64>,
    next_id: AtomicU64,
    tasks: BTreeMap<u64, Task>,
}

static mut EXECUTOR: Executor = Executor::new();

impl Executor {
    pub const fn new() -> Self {
        Self {
            tasks_to_poll: VecDeque::new(),
            next_id: AtomicU64::new(1),
            tasks: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let task = Task {
            future: Box::pin(future),
            waker: new_waker(id),
        };
        self.tasks.insert(id, task);
        self.tasks_to_poll.push_back(id);
    }

    pub fn do_work(&mut self) {
        while let Some(id) = self.tasks_to_poll.pop_front() {
            let task = self.tasks.get_mut(&id).unwrap();
            let waker = task.waker.clone();
            let mut context = Context::from_waker(&waker);
            if let Poll::Ready(()) = task.future.as_mut().poll(&mut context) {
                self.tasks.remove(&id);
            }
        }
    }

    pub fn backoff(&mut self) {
        // self.dispatch_syscall(0, &Syscall::Yield);
    }
}

struct SyscallFuture<'a> {
    syscall_context: SyscallContext<'a>,
    sent: bool,
}

impl Future for SyscallFuture<'_> {
    type Output = SyscallReturn;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker_data = context_waker_data(cx);
        if !self.sent {
            let task_id = waker_data.task_id;
            syscall_future(&mut self.syscall_context, task_id);
        }

        let result = self.syscall_context.syscall_result;
        if let SyscallReturn::NotComplete = result {
            return Poll::Pending;
        }

        Poll::Ready(result)
    }
}

pub fn syscall<'a>(args: Syscall<'a>) -> impl Future<Output = SyscallReturn> + 'a {
    SyscallFuture {
        syscall_context: args.into(),
        sent: false,
    }
}

#[derive(Clone, Copy)]
struct WakerData {
    task_id: u64,
    executor: &'static Executor,
}

fn context_waker_data<'a>(context: &'a Context) -> &'a WakerData {
    unsafe { &*(context.waker().data() as *const WakerData) }
}

const WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(exec_clone, exec_wake, exec_wake_by_ref, exec_drop);

fn exec_clone(data: *const ()) -> RawWaker {
    let data = unsafe { Arc::from_raw(data as *const WakerData) };
    let wd = Arc::into_raw(data.clone());
    let _ = Arc::into_raw(data);
    RawWaker::new(wd as *const (), &WAKER_VTABLE)
}

unsafe fn exec_wake(data: *const ()) {
    let data = unsafe { &*(data as *const WakerData) };
    EXECUTOR.tasks_to_poll.push_back(data.task_id);
}

unsafe fn exec_wake_by_ref(data: *const ()) {
    exec_wake(data)
}

unsafe fn exec_drop(data: *const ()) {
    let _ = Arc::from_raw(data as *mut WakerData);
}

fn new_waker(task_id: u64) -> Waker {
    let data = Arc::new(WakerData { task_id, executor: unsafe { &EXECUTOR }  });
    let data = Arc::into_raw(data);
    let raw_waker = RawWaker::new(data as *const (), &WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

pub unsafe fn spawn(task: impl Future<Output = ()> + Sized + 'static) {
    EXECUTOR.spawn(task);
}

pub unsafe fn run() {
    loop {
        // syscall::print("executor looping\n");
        EXECUTOR.do_work();

        // if task 1 ended, we're done
        if EXECUTOR.tasks.get(&1).is_none() {
            break;
        }

        EXECUTOR.backoff();
    }
}

// pub fn dispatch_syscall(args: &Syscall) -> SyscallReturn {
//     unsafe { EXECUTOR.dispatch_syscall(0, args) }
// }
