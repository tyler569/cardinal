use crate::syscall;
use core::fmt::Write;

pub struct SyscallPrint;

impl Write for SyscallPrint {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        syscall::print(s);
        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    let _ = SyscallPrint.write_fmt(args).map_err(|err| {
        panic!("print error: {}", err);
    });
}

#[macro_export]
macro_rules! print {
    () => ();
    ($($arg:tt)*) => ({
        $crate::format::_print(format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! println {
    () => ({ print!("\n"); });
    ($fmt:expr) => ({ print!(concat!($fmt, "\n")); });
    ($fmt:expr, $($arg:tt)*) => ({ print!(concat!($fmt, "\n"), $($arg)*); });
}
