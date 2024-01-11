macro_rules! format {
    ($($arg:tt)*) => ({
        let mut s = alloc::string::String::new();
        core::fmt::write(&mut s, format_args!($($arg)*)).unwrap();
        s
    })
}

macro_rules! print {
    ($($arg:tt)*) => ({
        let s = format!($($arg)*);
        $crate::syscall::println(&s);
    })
}

pub(crate) use {format, print};