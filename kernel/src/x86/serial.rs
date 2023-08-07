use crate::x86::pio;
use alloc::collections::VecDeque;
use core::future::Future;
use core::task::Waker;
use spin::{Lazy, Mutex};
use crate::print::println;

pub struct SerialPort {
    port: u16,
    queue: Mutex<VecDeque<u8>>,
    waker: Mutex<Option<Waker>>,
}

impl SerialPort {
    unsafe fn init(&mut self) {
        pio::write_u8(self.port + 1, 0x00); // Disable all interrupts
        pio::write_u8(self.port + 3, 0x80); // Enable DLAB (set baud rate divisor)
        pio::write_u8(self.port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        pio::write_u8(self.port + 1, 0x00); //                  (hi byte)
        pio::write_u8(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
        pio::write_u8(self.port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        pio::write_u8(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }

    pub unsafe fn new(port: u16) -> Self {
        let mut sp = SerialPort {
            port,
            queue: Mutex::new(VecDeque::new()),
            waker: Mutex::new(None),
        };
        sp.init();
        sp
    }

    pub unsafe fn handle_interrupt(&mut self) {
        let mut queue = self.queue.lock();
        while pio::read_u8(self.port + 5) & 1 != 0 {
            let b = pio::read_u8(self.port);
            queue.push_back(b);
        }
        if let Some(waker) = self.waker.lock().take() {
            waker.wake();
        }
    }
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            for b in s.bytes() {
                while pio::read_u8(self.port + 5) & 0x20 == 0 {}
                pio::write_u8(self.port, b);
            }
        }
        Ok(())
    }
}

impl Future for SerialPort {
    type Output = u8;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let mut queue = self.queue.lock();
        if let Some(b) = queue.pop_front() {
            return core::task::Poll::Ready(b);
        }
        let mut waker = self.waker.lock();
        *waker = Some(cx.waker().clone());
        core::task::Poll::Pending
    }
}

pub static SERIAL: Lazy<Mutex<SerialPort>> =
    Lazy::new(|| unsafe { Mutex::new(SerialPort::new(0x3F8)) });
