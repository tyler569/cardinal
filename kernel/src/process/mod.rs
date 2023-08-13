use crate::arch::{Context, PageTable};
use crate::print::println;
use crate::vmm::PageFlags;
use crate::{arch, pmm, vmm};
use bitflags::Flags;
use elf::endian::LittleEndian;

mod spawn;

#[derive(Debug)]
pub struct Process {
    context: arch::Context,
    vm_root: *mut PageTable,
    state: ProcessState,
    pending_signals: u64,
    exit_code: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ProcessState {
    Running,
    Waiting,
    Exited,
}

impl Process {
    pub unsafe fn new(elf_data: &[u8]) -> Self {
        let efile = elf::ElfBytes::<LittleEndian>::minimal_parse(elf_data).unwrap();
        let vm_root = arch::new_tree();
        let context = Context::new_user(efile.ehdr.e_entry as usize);
        let base = elf_data.as_ptr() as usize;

        for ph in efile.segments().unwrap() {
            if ph.p_type == elf::abi::PT_LOAD {
                let mut flags: PageFlags = PageFlags::READ | PageFlags::USER;
                if ph.p_flags & elf::abi::PF_W != 0 {
                    flags |= PageFlags::WRITE;
                }
                if ph.p_flags & elf::abi::PF_X != 0 {
                    flags |= PageFlags::EXECUTE;
                }
                for i in 0..(ph.p_memsz as usize + 0xfff) / 0x1000 {
                    let page_vma = base + ph.p_offset as usize + i * 0x1000;
                    let page_phy = arch::physical_address(page_vma).unwrap() & !0xfff;
                    let mapped_vma = (ph.p_vaddr as usize + i * 0x1000) & !0xfff;

                    // println!("map {:x} to {:x} with flags {:?}", page_phy, mapped_vma, flags);
                    arch::map_in_table(vm_root, mapped_vma, page_phy, flags);
                }
            }
        }

        arch::map_in_table(
            vm_root,
            0x7FFF_FF00_0000,
            pmm::alloc().unwrap(),
            PageFlags::READ | PageFlags::WRITE | PageFlags::USER,
        );

        Self {
            context,
            vm_root,
            state: ProcessState::Running,
            pending_signals: 0,
            exit_code: None,
        }
    }

    pub fn start(&mut self) -> ! {
        unsafe {
            // arch::print_page_table(self.vm_root);
            arch::load_tree(self.vm_root);
            arch::long_jump_usermode(self.context.ip(), arch::USER_STACK_TOP)
        }
    }
}
