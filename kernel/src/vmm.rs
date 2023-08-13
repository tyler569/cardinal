use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct PageFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const USER = 1 << 3;
    }
}
