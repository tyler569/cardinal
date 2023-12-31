use crate::net::Packet;
use crate::per_cpu::PerCpu;
use crate::print::println;
use crate::{arch, process};
use alloc::collections::{BTreeMap, VecDeque};
use cardinal3_interface::Error;
use core::cmp::min;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use spin::Mutex;

pub struct Socket {
    id: u64,
    dgs: Mutex<VecDeque<Packet>>,
    futures: Mutex<VecDeque<Waker>>,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

impl Socket {
    pub fn new() -> u64 {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let sock = Self {
            id,
            dgs: Mutex::new(VecDeque::new()),
            futures: Mutex::new(VecDeque::new()),
        };

        ALL.lock().insert(id, sock);
        id
    }

    pub fn write(&self, packet: Packet) -> usize {
        let len = packet.data.len();
        self.dgs.lock().push_back(packet);
        self.futures.lock().drain(..).for_each(|waker| waker.wake());
        len
    }

    pub fn read<'a>(&self, buffer: *mut [u8]) -> SocketRead {
        SocketRead {
            socket_id: self.id,
            process_id: PerCpu::running().unwrap(),
            buffer,
        }
    }
}

#[must_use = "futures do nothing if not polled or awaited"]
pub struct SocketRead {
    socket_id: u64,
    process_id: u64,
    buffer: *mut [u8],
}

impl Future for SocketRead {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let binding = ALL.lock();
        let socket = binding.get(&self.socket_id).unwrap();
        let x = match socket.dgs.lock().pop_front() {
            Some(packet) => {
                let len = min(packet.data.len(), self.buffer.len());
                let source = &packet.data[..len];
                let Some(tree) = process::with(self.process_id, |p| p.vm_root()) else {
                    // the process that was trying to read from this socket no longer exists
                    return Poll::Ready(0);
                };
                arch::load_tree(tree);
                unsafe { (&mut *self.buffer)[..len].copy_from_slice(source) };
                Poll::Ready(len)
            }
            None => {
                socket.futures.lock().push_back(cx.waker().clone());
                Poll::Pending
            }
        };
        x
    }
}

pub static ALL: Mutex<BTreeMap<u64, Socket>> = Mutex::new(BTreeMap::new());

pub fn read(sn: u64, buf: *mut [u8]) -> Result<u64, Error> {
    let task = {
        let binding = ALL.lock();
        let Some(socket) = binding.get(&sn) else {
            return Err(Error::EINVAL);
        };
        socket.read(buf)
    };
    let pid = PerCpu::running().unwrap();
    crate::executor::spawn(async move {
        task.await;
        println!("[KERNEL: read completed]");
    });
    Ok(0)
}

pub fn write(sn: u64, buf: &[u8]) -> Result<u64, Error> {
    let binding = ALL.lock();
    let Some(socket) = binding.get(&sn) else {
        return Err(Error::EINVAL);
    };
    let packet = Packet::new(buf);
    Ok(socket.write(packet) as u64)
}
