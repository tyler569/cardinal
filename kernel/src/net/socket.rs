use crate::net::Packet;
use crate::per_cpu::PerCpu;
use crate::print::println;
use crate::{arch, process};
use alloc::collections::{BTreeMap, VecDeque};
use cardinal3_interface::{Error, SyscallReturn};
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
}

pub static ALL: Mutex<BTreeMap<u64, Socket>> = Mutex::new(BTreeMap::new());

pub fn read(sn: u64, buf: &&mut [u8]) -> SyscallReturn {
    SyscallReturn::NotComplete
}

pub fn write(sn: u64, buf: &[u8]) -> SyscallReturn {
    SyscallReturn::NotComplete
}
