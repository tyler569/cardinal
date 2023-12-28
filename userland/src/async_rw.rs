use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use cardinal3_interface::{Error, TaskId};
use crate::syscall;

pub trait Read {
    fn read<'obj, 'buf>(&'obj mut self, buf: &'buf mut [u8]) -> ReadFuture<'buf>;
}

#[derive(Clone, Copy, Debug)]
struct File(u64);

impl Read for File {
    fn read<'obj, 'buf>(&'obj mut self, buf: &'buf mut [u8]) -> ReadFuture<'buf> {
        ReadFuture {
            file_number: self.0,
            buf,
        }
    }
}

pub struct ReadFuture<'a> {
    file_number: u64,
    buf: &'a mut [u8],
}

impl Future for ReadFuture<'_> {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let result = syscall::async_read_2(self.file_number, self.buf, TaskId(0));
            return if result == Error::EAGAIN as u64 {
                Poll::Pending
            } else {
                Poll::Ready(result as usize)
            }
        }
    }
}