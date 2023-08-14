use alloc::collections::{BTreeMap, VecDeque};
use core::arch::asm;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::arch::{Context, PageTable};
use crate::print::println;
use crate::vmm::PageFlags;
use crate::{arch, pmm, vmm};
use bitflags::Flags;
use elf::endian::LittleEndian;
use spin::Mutex;
use crate::per_cpu::PerCpu;

mod spawn;

#[derive(Debug)]
pub struct Process {
    pub context: arch::Context,
    pub vm_root: *mut PageTable,
    pub state: ProcessState,
    pub pending_signals: u64,
    pub exit_code: Option<u32>,
    pub id: usize,
}

// Rust is mad because of the PageTable, but we'll never modify that through this object
// in an unsynchronized way.
unsafe impl Send for Process {}
unsafe impl Sync for Process {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessState {
    Running,
    Waiting,
    Exited,
}

static DO_IT_ONCE: AtomicBool = AtomicBool::new(true);

impl Process {
    pub unsafe fn new(elf_data: &[u8]) -> usize {
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

        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        let process = Self {
            context,
            vm_root,
            state: ProcessState::Running,
            pending_signals: 0,
            exit_code: None,
            id,
        };

        ALL.lock().insert(id, process);
        println!("Created process {}", id);
        id
    }

    pub fn run(id: usize) -> ! {
        let context = unsafe  {
            let mut binding = ALL.lock();
            let process = binding.get_mut(&id).unwrap();
            // arch::print_page_table(self.vm_root);
            PerCpu::get_mut().running = Some(NonNull::new_unchecked(process as *mut _));
            arch::load_tree(process.vm_root);
            &process.context as *const _
        };
        unsafe {
            arch::long_jump_context(context)
        }
    }
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
pub static ALL: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
pub static RUNNABLE: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::new());

pub fn schedule(proc: &Process) {
    RUNNABLE.lock().push_back(proc.id);
}

pub fn schedule_pid(pid: usize) {
    RUNNABLE.lock().push_back(pid);
}
