use crate::print::{print, println};
use crate::{arch, timer};
use core::future::Future;
use core::pin::{pin, Pin};
use core::task::Poll::{Pending, Ready};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use core::time::Duration;

pub async fn foobar(a: i32, b: i32) -> i32 {
    incbar(a).await + b
}
pub async fn incbar(a: i32) -> i32 {
    slpbar(a).await + 1
}
pub async fn slpbar(a: i32) -> i32 {
    sleep().await;
    a
}

const MY_VTABLE: RawWakerVTable = RawWakerVTable::new(my_clone, my_wake, my_wake_by_ref, my_drop);

pub fn run_async<T>(future: impl Future<Output = T>) -> T {
    let raw_waker = RawWaker::new(&(), &MY_VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut context = Context::from_waker(&waker);

    let mut pinned = pin!(future);
    let mut result = pinned.as_mut().poll(&mut context);

    while result.is_pending() {
        result = pinned.as_mut().poll(&mut context);
    }

    let Ready(value) = result else {
        unreachable!();
    };

    value
}

fn my_clone(data: *const ()) -> RawWaker {
    println!("clone called!");
    RawWaker::new(&(), &MY_VTABLE)
}

unsafe fn my_wake(data: *const ()) {
    println!("wake called!")
}

unsafe fn my_wake_by_ref(data: *const ()) {
    println!("wake_by_ref called!")
}

unsafe fn my_drop(data: *const ()) {
    println!("drop called!")
}

struct Sleep {
    start_ts: u64,
    until: u64,
}

impl Sleep {
    fn new(duration: core::time::Duration) -> Self {
        let start_ts = timer::timestamp();
        Self {
            start_ts,
            until: start_ts + duration.as_millis() as u64,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if timer::timestamp() > self.until {
            Ready(())
        } else {
            Pending
        }
    }
}

fn sleep() -> Sleep {
    Sleep::new(Duration::from_secs(1))
}
