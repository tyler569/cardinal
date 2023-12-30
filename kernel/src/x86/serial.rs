use crate::print::{print, println};
use crate::x86::pio;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::fmt::Write;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use spin::{Lazy, Mutex, MutexGuard};

pub struct SerialPort {
    port: u16,
    writer: Mutex<SerialPortWriter>,
    queue: Mutex<VecDeque<u8>>,
    futures: Mutex<Vec<Waker>>,
}

impl SerialPort {
    unsafe fn init(&mut self) {
        pio::write_u8(self.port + 1, 0x00); // Disable all interrupts
        pio::write_u8(self.port + 3, 0x80); // Enable DLAB (set baud rate divisor)
        pio::write_u8(self.port + 0, 0x01); // Set divisor to 1 (lo byte) 115200 baud
        pio::write_u8(self.port + 1, 0x00); //                  (hi byte)
        pio::write_u8(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
        pio::write_u8(self.port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        pio::write_u8(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
        pio::write_u8(self.port + 1, 0x01); // Enable Data Available interrupt
    }

    pub unsafe fn new(port: u16) -> Self {
        let mut sp = SerialPort {
            port,
            queue: Mutex::new(VecDeque::new()),
            futures: Mutex::new(Vec::new()),
            writer: Mutex::new(SerialPortWriter { port }),
        };
        sp.init();
        sp
    }

    pub unsafe fn handle_interrupt(&self) {
        while pio::read_u8(self.port + 5) & 1 != 0 {
            let b = pio::read_u8(self.port);
            self.queue.lock().push_back(b);
        }
        for waker in self.futures.lock().drain(..) {
            waker.wake();
        }
    }

    pub fn write(&self) -> MutexGuard<SerialPortWriter> {
        self.writer.lock()
    }

    pub fn try_write(&self) -> Option<MutexGuard<SerialPortWriter>> {
        let x = self.writer.try_lock();

        // if x.is_none() {
        //     let mut spw = SerialPortWriter { port: self.port };
        //     spw.write_str("XXX").unwrap();
        // }

        x
    }

    pub fn read(&self) -> SerialPortReadFuture {
        SerialPortReadFuture { port: self.port }
    }
}

pub struct SerialPortWriter {
    port: u16,
}

impl SerialPortWriter {
    fn write_byte(&mut self, b: u8) {
        unsafe {
            while pio::read_u8(self.port + 5) & 0x20 == 0 {}
            pio::write_u8(self.port, b);
        }
    }
}

impl Write for SerialPortWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            for b in s.bytes() {
                if b == b'\n' {
                    self.write_byte(b'\r');
                }
                self.write_byte(b);
            }
        }
        Ok(())
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SerialPortReadFuture {
    port: u16,
}

impl Future for SerialPortReadFuture {
    type Output = u8;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut queue = SERIAL.queue.lock();
        match queue.pop_front() {
            Some(b'\r') => Poll::Ready(b'\n'),
            Some(0x7f) => Poll::Ready(0x08),
            Some(b) => Poll::Ready(b),
            None => {
                SERIAL.futures.lock().push(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

pub static SERIAL: Lazy<SerialPort> = Lazy::new(|| unsafe { SerialPort::new(0x3F8) });
