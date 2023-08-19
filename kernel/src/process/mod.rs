use crate::arch::{Context, PageTable};
use crate::per_cpu::PerCpu;
use crate::print::println;
use crate::vmm::PageFlags;
use crate::{arch, pmm, vmm};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use bitflags::Flags;
use core::arch::asm;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use elf::endian::LittleEndian;
use spin::Mutex;

mod spawn;

#[derive(Debug)]
pub struct Process {
    pub context: arch::Context,
    pub vm_root: *mut PageTable,
    pub state: ProcessState,
    pub pending_signals: u64,
    pub exit_code: Option<u32>,
    pub id: usize,
    pub sched_in: u64,
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
    pub unsafe fn new(elf_data: &[u8], arg: usize) -> usize {
        let efile = elf::ElfBytes::<LittleEndian>::minimal_parse(elf_data).unwrap();
        let vm_root = arch::new_tree();
        let mut context = Context::new_user(efile.ehdr.e_entry as usize);
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

        for i in 0..16 {
            arch::map_in_table(
                vm_root,
                0x7FFF_FFF0_0000 + 0x1000 * i,
                pmm::alloc().unwrap(),
                PageFlags::READ | PageFlags::WRITE | PageFlags::USER,
            )
        }

        context.set_arg1(arg as u64);

        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        let process = Self {
            context,
            vm_root,
            state: ProcessState::Running,
            pending_signals: 0,
            exit_code: None,
            id,
            sched_in: 0,
        };

        ALL.lock().insert(id, process);
        println!("[cpu:{} created pid:{}]", arch::cpu_num(), id);
        id
    }

    pub fn run(id: usize) -> ! {
        println!("[cpu:{} running pid:{}]", arch::cpu_num(), id);
        let context = unsafe {
            let mut binding = ALL.lock();
            let process = binding.get_mut(&id).unwrap();
            process.sched_in = PerCpu::ticks();
            PerCpu::set_running(Some(id));
            arch::load_tree(process.vm_root);
            process.context.clone()
        };
        unsafe { arch::long_jump_context(&context) }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        arch::free_tree(self.vm_root);
        println!("[cpu:{} dropped pid:{}]", arch::cpu_num(), self.id);
    }
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
pub static ALL: Mutex<BTreeMap<usize, Process>> = Mutex::new(BTreeMap::new());
pub static RUNNABLE: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::new());

pub fn schedule(proc: &Process) {
    schedule_pid(proc.id);
}

pub fn schedule_pid(pid: usize) {
    let mut handle = RUNNABLE.lock();
    // assert!(handle.iter().all(|&p| p != pid), "double scheduling {}!", pid);
    if handle.iter().any(|&p| p == pid) {
        return;
    }
    handle.push_back(pid);
}

pub fn run_usermode_program() {
    let Some(pid) = RUNNABLE.lock().pop_front() else {
        return;
    };
    Process::run(pid)
}
