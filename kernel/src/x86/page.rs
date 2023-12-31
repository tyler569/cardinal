use crate::print::{print, println};
use crate::vmm::PageFlags;
use crate::{pmm, x86};
use core::arch::asm;
use core::fmt::{Display, Formatter};
use spin::Once;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Pte(pub u64);

#[allow(dead_code)] // modelling real-world
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
    pub const NX: u64 = 0x8000_0000_0000_0000;

    pub const P4_MASK: u64 = 0xffff_ff80_0000_0000;
    pub const P3_MASK: u64 = 0xffff_ffff_c000_0000;
    pub const P2_MASK: u64 = 0xffff_ffff_ffe0_0000;
    pub const P1_MASK: u64 = 0xffff_ffff_ffff_f000;

    pub const P1_OFFSET: u64 = !Self::P1_MASK;
    pub const P2_OFFSET: u64 = !Self::P2_MASK;
    pub const P3_OFFSET: u64 = !Self::P3_MASK;
    pub const P4_OFFSET: u64 = !Self::P4_MASK;

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

    pub fn is_nx(self) -> bool {
        self.is_present() && self.0 & Self::NX != 0
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

impl Display for Pte {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Pte({:#018x}, {}{}{}{}{}{}{}{})",
            self.address(),
            if self.is_writeable() { "W" } else { "-" },
            if self.is_usermode() { "U" } else { "-" },
            if self.is_accessed() { "A" } else { "-" },
            if self.is_dirty() { "D" } else { "-" },
            if self.is_huge() { "H" } else { "-" },
            if self.is_global() { "G" } else { "-" },
            if self.is_copy_on_write() { "C" } else { "-" },
            if self.is_nx() { "X" } else { "-" },
        )
    }
}

fn generic_flags(flags: PageFlags) -> u64 {
    let mut x86_flags = Pte::NX | Pte::PRESENT;
    if !flags.contains(PageFlags::READ) {
        print!("warning: x86_64 cannot create non-readable pages\n")
    }
    if flags.contains(PageFlags::WRITE) {
        x86_flags |= Pte::WRITEABLE;
    }
    if flags.contains(PageFlags::EXECUTE) {
        x86_flags &= !Pte::NX;
    }
    if flags.contains(PageFlags::USER) {
        x86_flags |= Pte::USERMODE;
    }
    x86_flags
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [Pte; 512],
}

pub fn get_vm_root() -> *mut PageTable {
    let vm_root: u64;
    unsafe {
        asm!(
            "mov {}, cr3",
            out(reg) vm_root,
        );
    }
    x86::direct_map_mut((vm_root & 0xffff_ffff_ffff_f000) as *mut PageTable)
}

#[allow(dead_code)] // debug
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
            if level > 1 && !entry.is_huge() {
                println!("Table {}: {}", level - 1, entry);
                print_page_table_level(entry.next_table(), level - 1, addr);
            } else {
                println!("Entry {:#018x}: {} ({:#018x})", addr, entry, entry.0);
            }
        }
    }
}

pub unsafe fn map_in_table(root: *mut PageTable, virt: usize, phys: u64, flags: PageFlags) {
    let flags = generic_flags(flags) | Pte::PRESENT;

    let p4_offset = (virt >> 39) & 0x1ff;
    let p3_offset = (virt >> 30) & 0x1ff;
    let p2_offset = (virt >> 21) & 0x1ff;
    let p1_offset = (virt >> 12) & 0x1ff;

    let table_flags = if virt < 0xffff_8000_0000_0000 {
        Pte::PRESENT | Pte::WRITEABLE | Pte::USERMODE
    } else {
        Pte::PRESENT | Pte::WRITEABLE
    };

    let p4 = &mut (*root).entries[p4_offset];
    if !p4.is_present() {
        let p3_page = pmm::alloc().unwrap();
        let p3_ptr = x86::direct_map_offset(p3_page) as *mut PageTable;
        p4.set(p3_page, table_flags);
        for entry in (*p3_ptr).entries.iter_mut() {
            entry.set(0, 0);
        }
    }

    let p3 = &mut (*p4.next_table_mut()).entries[p3_offset];
    if !p3.is_present() {
        let p2_page = pmm::alloc().unwrap();
        let p2_ptr = x86::direct_map_offset(p2_page) as *mut PageTable;
        p3.set(p2_page, table_flags);
        for entry in (*p2_ptr).entries.iter_mut() {
            entry.set(0, 0);
        }
    }
    if p3.is_huge() {
        panic!("tried to map inside a huge page")
    }

    let p2 = &mut (*p3.next_table_mut()).entries[p2_offset];
    if !p2.is_present() {
        let p1_page = pmm::alloc().unwrap();
        let p1_ptr = x86::direct_map_offset(p1_page) as *mut PageTable;
        p2.set(p1_page, table_flags);
        for entry in (*p1_ptr).entries.iter_mut() {
            entry.set(0, 0);
        }
    }
    if p2.is_huge() {
        panic!("tried to map inside a huge page")
    }

    let p1 = &mut (*p2.next_table_mut()).entries[p1_offset];
    p1.set(phys, flags);
}

pub fn physical_address(virtual_address: usize) -> Option<u64> {
    let p4_offset = (virtual_address >> 39) & 0x1ff;
    let p3_offset = (virtual_address >> 30) & 0x1ff;
    let p2_offset = (virtual_address >> 21) & 0x1ff;
    let p1_offset = (virtual_address >> 12) & 0x1ff;

    let p4 = unsafe { &(*get_vm_root()).entries[p4_offset] };
    if !p4.is_present() {
        return None;
    }

    let p3 = unsafe { &(*p4.next_table()).entries[p3_offset] };
    if !p3.is_present() {
        return None;
    }
    if p3.is_huge() {
        return Some(p3.address() + (virtual_address as u64 & Pte::P3_OFFSET));
    }

    let p2 = unsafe { &(*p3.next_table()).entries[p2_offset] };
    if !p2.is_present() {
        return None;
    }
    if p2.is_huge() {
        return Some(p2.address() + (virtual_address as u64 & Pte::P2_OFFSET));
    }

    let p1 = unsafe { &(*p2.next_table()).entries[p1_offset] };
    if !p1.is_present() {
        return None;
    }

    Some(p1.address() + (virtual_address as u64 & Pte::P1_OFFSET))
}

pub fn new_tree() -> *mut PageTable {
    let root = get_vm_root();

    let page = pmm::alloc().unwrap();
    let page = x86::direct_map_offset(page) as *mut PageTable;
    unsafe {
        for (i, entry) in (*page).entries.iter_mut().enumerate() {
            if i < 256 {
                entry.set(0, 0);
            } else {
                *entry = (*root).entries[i];
            }
        }
    }
    page
}

pub fn load_tree(root: *const PageTable) {
    unsafe {
        let root = x86::physical_address(root as usize).unwrap();
        asm!("mov cr3, {}", in(reg) root);
    }
}

pub fn free_tree(root: *mut PageTable) {
    load_tree(*KERNEL_ROOT.get().unwrap() as *const PageTable);
    unsafe {
        free_tree_level(root, 4);
        pmm::free(x86::physical_address(root as usize).unwrap());
    }
}

unsafe fn free_tree_level(root: *mut PageTable, level: usize) {
    for (i, entry) in (*root).entries.iter().enumerate() {
        if level == 4 && i > 255 {
            break;
        }
        if entry.is_present() {
            if level == 1 {
                pmm::free(entry.address());
            } else {
                free_tree_level(
                    x86::direct_map_offset(entry.address()) as *mut PageTable,
                    level - 1,
                );
                pmm::free(entry.address());
            }
        }
    }
}

static KERNEL_ROOT: Once<usize> = Once::new();

pub unsafe fn init() {
    let root = &mut *get_vm_root();
    root.entries[0].set(0, 0);
    root.entries[1].set(0, 0);
    root.entries[257].set(0, 0);
    KERNEL_ROOT.call_once(|| root as *const _ as usize);
}
/*
bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Entry(u64);
pub struct Table([Entry; 512]);

pub enum EntryValue {
    NextTable(u64),
    Frame(u64, usize),
    Nothing,
}

impl Entry {
    pub fn flags(self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn value(self) -> Option<u64> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(self.0 & 0x000fffff_fffff000)
        } else {
            None
        }
    }
}

fn addr_to_table(addr: u64) -> &'static mut Table {
    unsafe { &mut *(arch::direct_map_offset(addr) as *mut Table) }
}

fn resolve_one(table: &Table, virtual_address: usize, level: i32) -> EntryValue {
    let index = (virtual_address >> (12 + 9 * level)) & 0o777;
    let entry_flags = table.0[index].flags();
    return if !entry_flags.contains(EntryFlags::PRESENT) {
        EntryValue::Nothing
    } else if level == 0 || entry_flags.contains(EntryFlags::HUGE_PAGE) {
        let frame_address = table.0[index].value().unwrap();
        let frame_mask = if entry_flags.contains(EntryFlags::HUGE_PAGE) {
            0x0001_0000_0000_0000 - (1 << 12 + 9 * level)
        } else {
            0x0000_ffff_ffff_f000
        };
        EntryValue::Frame(table.0[index].value().unwrap(), frame_mask)
    } else {
        let next_table_address = table.0[index].value().unwrap();
        EntryValue::NextTable(next_table_address)
    }
}

pub fn resolve(root: &Table, virtual_address: usize) -> EntryValue {
    let mut table = root;
    let mut value = EntryValue::Nothing;
    for level in (0..4).rev() {
        match resolve_one(table, virtual_address, level) {
            EntryValue::NextTable(address) => {
                table = addr_to_table(address);
            }
            EntryValue::Frame(address, mask) => {
                value = EntryValue::Frame(address, mask);
            }
            EntryValue::Nothing => {
                break;
            }
        }
    }
    return value;
}
 */
