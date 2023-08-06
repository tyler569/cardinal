#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct PTE(pub u64);

impl PTE {
    pub const PRESENT: u64 = 0x01;
    pub const WRITEABLE: u64 = 0x02;
    pub const USERMODE: u64 = 0x04;
    pub const ACCESSED: u64 = 0x20;
    pub const DIRTY: u64 = 0x40;
    pub const IS_HUGE: u64 = 0x80;
    pub const GLOBAL: u64 = 0x100;
    pub const COPY_ON_WRITE: u64 = 0x200;
    pub const OS_RESERVED2: u64 = 0x400;
    pub const OS_RESERVED3: u64 = 0x800;

    pub fn is_present(self) -> bool {
        self.0 & Self::PRESENT != 0
    }

    pub fn is_writeable(self) -> bool {
        self.is_present() && self.0 & Self::WRITEABLE != 0
    }

    pub fn is_usermode(self) -> bool {
        self.is_present() && self.0 & Self::USERMODE != 0
    }

    pub fn is_accessed(self) -> bool {
        self.is_present() && self.0 & Self::ACCESSED != 0
    }

    pub fn is_dirty(self) -> bool {
        self.is_present() && self.0 & Self::DIRTY != 0
    }

    pub fn is_huge(self) -> bool {
        self.is_present() && self.0 & Self::IS_HUGE != 0
    }

    pub fn is_global(self) -> bool {
        self.is_present() && self.0 & Self::GLOBAL != 0
    }

    pub fn is_copy_on_write(self) -> bool {
        self.is_present() && self.0 & Self::COPY_ON_WRITE != 0
    }

    pub fn new(address: u64, flags: u64) -> Self {
        Self(address | flags)
    }

    pub fn address(self) -> u64 {
        self.0 & 0x000fffff_fffff000
    }

    pub fn flags(self) -> u64 {
        self.0 & 0xfff00000_00000fff
    }

    pub fn set_address(&mut self, address: u64) {
        self.0 = (self.0 & 0xfff00000_00000fff) | address;
    }

    pub fn set_flags(&mut self, flags: u64) {
        self.0 = (self.0 & 0x000fffff_fffff000) | flags;
    }

    pub fn set(&mut self, address: u64, flags: u64) {
        self.0 = address | flags;
    }

    pub fn next_table(&self) -> *const PageTable {
        core::ptr::null()
    }

    pub fn next_table_mut(&mut self) -> *mut PageTable {
        core::ptr::null_mut()
    }
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [PTE; 512],
}
