use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::per_cpu::PerCpu;

struct TimerEvent {
    callback: Box<dyn FnOnce()>,
}

pub struct Timer {
    ticks: AtomicU64,
    events: BTreeMap<(u64, u64), TimerEvent>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            ticks: AtomicU64::new(0),
            events: BTreeMap::new(),
        }
    }

    pub fn raw_insert(&mut self, time: u64, callback: Box<dyn FnOnce()>) {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let event = TimerEvent {
            callback,
        };

        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        self.events.insert((time, id), event);
    }

    fn duration_to_ticks(duration: core::time::Duration) -> u64 {
        duration.as_millis() as u64
    }

    pub fn insert(&mut self, duration: core::time::Duration, callback: Box<dyn FnOnce()>) {
        let time = self.ticks.load(Ordering::SeqCst) + Self::duration_to_ticks(duration);
        self.raw_insert(time, callback);
    }

    pub fn tick(&mut self) {
        self.ticks.fetch_add(1, Ordering::SeqCst);
        let up_to = self.ticks.load(Ordering::SeqCst);

        let mut keys = Vec::new();

        for (time, id) in self.events.keys() {
            if *time <= up_to {
                keys.push((*time, *id));
            }
        }

        for key in keys {
            let event = self.events.remove(&key).unwrap();
            (event.callback)();
        }
    }
}

pub fn timestamp() -> u64 {
    PerCpu::get().timer.ticks.load(Ordering::Relaxed)
}

pub fn insert<F: FnOnce() + 'static>(duration: core::time::Duration, callback: F) {
    let callback = Box::new(callback);
    PerCpu::get_mut().timer.insert(duration, callback);
}