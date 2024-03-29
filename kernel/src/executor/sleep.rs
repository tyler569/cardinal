use crate::per_cpu::PerCpu;
use crate::timer;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct Sleep {
    until: u64,
}

impl Sleep {
    fn new(duration: core::time::Duration) -> Self {
        Self {
            until: PerCpu::ticks() + timer::ticks_for(duration),
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if PerCpu::ticks() > self.until {
            Poll::Ready(())
        } else {
            let waker = cx.waker().clone();
            timer::insert_at(self.until, move || waker.wake_by_ref());
            Poll::Pending
        }
    }
}

pub fn sleep(duration: core::time::Duration) -> impl Future<Output = ()> {
    Sleep::new(duration)
}
