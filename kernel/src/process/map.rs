use core::cmp::min;
use elf::ElfBytes;
use elf::endian::LittleEndian;
use elf::segment::ProgramHeader;
use crate::arch::PageTable;
use crate::{arch, pmm};
use crate::vmm::PageFlags;

pub unsafe fn map_elf_into_address_space(elf_data: &[u8], vm_root: *mut PageTable) -> ElfBytes<LittleEndian> {
    let base = elf_data.as_ptr() as usize;
    let elf = ElfBytes::minimal_parse(elf_data).expect("Invalid elf");

    for ph in elf.segments().unwrap() {
        if ph.p_type == elf::abi::PT_LOAD {
            let mut flags: PageFlags = PageFlags::READ | PageFlags::USER;
            if ph.p_flags & elf::abi::PF_W != 0 {
                create_rw_mapping(&ph, base, vm_root);
            } else {
                create_ro_mapping(&ph, base, vm_root);
            }
        }
    }

    for i in 0..arch::USER_STACK_PAGES {
        arch::map_in_table(
            vm_root,
            arch::USER_STACK_BASE + arch::PAGE_SIZE * i,
            pmm::alloc().unwrap(),
            PageFlags::READ | PageFlags::WRITE | PageFlags::USER,
        )
    }

    elf
}

pub fn overlapping_pages(base: usize, len: usize) -> usize {
    let bottom = base & !arch::PAGE_MASK;
    let top = (base + len).next_multiple_of(arch::PAGE_SIZE);
    (top - bottom) / arch::PAGE_SIZE
}

pub fn mapping_pages(ph: &ProgramHeader, base: usize) -> impl Iterator<Item = (usize, usize)> {
    let number = overlapping_pages(ph.p_vaddr as usize, ph.p_memsz as usize);
    let file_base = (base + ph.p_offset as usize) & !arch::PAGE_MASK;
    let vaddr_base = ph.p_vaddr as usize & !arch::PAGE_MASK;

    (0..number).map(move |n| (
        file_base + n * arch::PAGE_SIZE,
        vaddr_base + n * arch::PAGE_SIZE,
    ))
}

pub unsafe fn create_ro_mapping(ph: &ProgramHeader, base: usize, vm_root: *mut PageTable) {
    let flags = if ph.p_flags & elf::abi::PF_X != 0 {
        PageFlags::READ | PageFlags::USER | PageFlags::EXECUTE
    } else {
        PageFlags::READ | PageFlags::USER
    };

    for (file_page, user_page) in mapping_pages(ph, base) {
        let file_phy = arch::physical_address(file_page).unwrap();

        arch::map_in_table(vm_root, user_page, file_phy, flags);
    }
}

pub unsafe fn create_rw_mapping(ph: &ProgramHeader, base: usize, vm_root: *mut PageTable) {
    let flags = PageFlags::READ | PageFlags::USER | PageFlags::WRITE;
    let copy_end = (ph.p_vaddr + ph.p_filesz) as usize;

    for (file_page, user_page) in mapping_pages(ph, base) {
        let copy_phy = pmm::alloc().unwrap();
        let copy_mapped = arch::direct_map_offset(copy_phy);

        let copy_len = min(arch::PAGE_SIZE, copy_end.saturating_sub(user_page));
        if copy_len > 0 {
            core::ptr::copy_nonoverlapping(file_page as *const u8, copy_mapped as *mut u8, copy_len);
        }
        let zero_len = arch::PAGE_SIZE - copy_len;
        if zero_len > 0 {
            core::ptr::write_bytes(copy_mapped as *mut u8, 0, zero_len);
        }

        arch::map_in_table(vm_root, user_page, copy_phy, flags);
    }
}