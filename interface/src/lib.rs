#![no_std]

// #[repr(u32)]
// pub enum Syscall {
//     Print = 0,
//     Exit = 1,
// }
//
// #[repr(C)]
// pub struct PrintArgs {
//     pub data: *const [u8],
// }
//
// #[repr(C)]
// pub struct ExitArgs {
//     pub code: u32,
// }

#[repr(C)]
pub enum Syscall<'a> {
    Print(&'a str),
    Exit(u32),
}