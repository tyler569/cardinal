use core::arch::asm;
use crate::print::println;
use crate::x86;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Pte(pub u64);

impl Pte {
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
        x86::direct_map(self.address() as *const PageTable)
    }

    pub fn next_table_mut(&mut self) -> *mut PageTable {
        self.next_table() as *mut PageTable
    }
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [Pte; 512],
}

pub fn get_vm_root() -> *mut PageTable {
    let vm_root;
    unsafe {
        asm!(
            "mov {}, cr3",
            out(reg) vm_root,
        );
    }
    x86::direct_map_mut(vm_root)
}

pub fn print_page_table(root: *const PageTable) {
    print_page_table_level(root, 4, 0);
}

fn print_page_table_level(root: *const PageTable, level: i32, addr: u64) {
    let root = unsafe { &*root };
    for (i, entry) in root.entries.iter().enumerate() {
        let mut addr = addr | (i as u64) << (12 + (level - 1) * 9);
        if addr & 0x0000_8000_0000_0000 != 0 {
            addr |= 0xFFFF_0000_0000_0000;
        }
        if entry.is_present() {
            println!("Entry {:#018x}: {:#012x}", addr, entry.address());
            if level > 1 && !entry.is_huge() {
                println!("Entry {:#018x}: {:#012x}", addr, entry.address());
                print_page_table_level(entry.next_table(), level - 1, addr);
            }
        }
    }
}

pub unsafe fn pte_ptr(root: *const PageTable, virt: usize) -> Option<*const Pte> {
    let root = unsafe { &*root };
    let p4_index = (virt >> 39) & 0x1ff;
    let p3_index = (virt >> 30) & 0x1ff;
    let p2_index = (virt >> 21) & 0x1ff;
    let p1_index = (virt >> 12) & 0x1ff;

    if !root.entries[p4_index].is_present() {
        return None;
    }

    let p3_table = root.entries[p4_index].next_table();
    if !(*p3_table).entries[p3_index].is_present() {
        return None;
    }

    let p2_table = (*p3_table).entries[p3_index].next_table();
    if !(*p2_table).entries[p2_index].is_present() {
        return None;
    }

    let p1_table = (*p2_table).entries[p2_index].next_table();
    Some(&(*p1_table).entries[p1_index])
}

pub unsafe fn pte_mut(root: *mut PageTable, virt: usize, create_flags: Option<u64>) -> Option<*mut Pte> {
    let mut root = unsafe { &mut *root };
    let p4_index = (virt >> 39) & 0x1ff;
    let p3_index = (virt >> 30) & 0x1ff;
    let p2_index = (virt >> 21) & 0x1ff;
    let p1_index = (virt >> 12) & 0x1ff;

    let Some(mut table_flags) = create_flags else {
        return pte_ptr(root, virt).map(|ptr| ptr as *mut Pte);
    };
    table_flags |= Pte::PRESENT;

    if !root.entries[p4_index].is_present() {
        let table = crate::pmm::alloc_zeroed().unwrap();
        root.entries[p4_index].set(table as u64, table_flags);
    }

    let p3_table = root.entries[p4_index].next_table_mut();
    if !(*p3_table).entries[p3_index].is_present() {
        let table = crate::pmm::alloc_zeroed().unwrap();
        (*p3_table).entries[p3_index].set(table as u64, table_flags);
    }

    let p2_table = (*p3_table).entries[p3_index].next_table_mut();
    if !(*p2_table).entries[p2_index].is_present() {
        let table = crate::pmm::alloc_zeroed().unwrap();
        (*p2_table).entries[p2_index].set(table as u64, table_flags);
    }

    let p1_table = (*p2_table).entries[p2_index].next_table_mut();
    if !(*p1_table).entries[p1_index].is_present() {
        let table = crate::pmm::alloc_zeroed().unwrap();
        (*p1_table).entries[p1_index].set(table as u64, table_flags);
    }

    Some(&mut (*p1_table).entries[p1_index])
}

pub unsafe fn map(root: *mut PageTable, virt: usize, phys: u64, flags: u64) {
    (*pte_mut(root, virt, Some(flags)).unwrap()).set(phys, flags);
}

pub fn physical_address(virtual_address: usize) -> Option<u64> {
    unsafe {
        let pte = pte_ptr(get_vm_root(), virtual_address)?;
        Some((*pte).address() + (virtual_address as u64 & 0xfff))
    }
}
