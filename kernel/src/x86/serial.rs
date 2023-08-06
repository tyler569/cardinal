use crate::x86::pio;
use spin::{Lazy, Mutex};

pub struct SerialPort {
    port: u16,
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
        let mut sp = SerialPort { port };
        sp.init();
        sp
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

pub static SERIAL: Lazy<Mutex<SerialPort>> =
    Lazy::new(|| unsafe { Mutex::new(SerialPort::new(0x3F8)) });
