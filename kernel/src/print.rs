pub fn _print(args: core::fmt::Arguments) {
    use crate::arch::SERIAL;
    use core::fmt::Write;
    SERIAL.lock().write_fmt(args).unwrap();
}

macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

macro_rules! println {
    () => ($crate::print::_print(format_args!("\n")));
    ($fmt:expr) => ($crate::print::_print(format_args!(concat!($fmt, "\r\n"))));
    ($fmt:expr, $($arg:tt)*) => ($crate::print::_print(format_args!(concat!($fmt, "\r\n"), $($arg)*)));
}

pub(crate) use {print, println};
