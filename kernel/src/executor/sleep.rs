use crate::print::println;
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
        // println!("[async] new sleep");
        Self {
            until: timer::timestamp() + timer::ticks_for(duration),
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if timer::timestamp() > self.until {
            // println!("[async] sleep done");
            Poll::Ready(())
        } else {
            // println!("[async] sleep not done");
            let waker = cx.waker().clone();
            timer::insert_at(self.until, move || waker.wake_by_ref());
            Poll::Pending
        }
    }
}

pub fn sleep(duration: core::time::Duration) -> impl Future<Output = ()> {
    Sleep::new(duration)
}
