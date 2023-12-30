use crate::arch::{Context, PageTable};
use crate::per_cpu::PerCpu;
use crate::print::println;
use crate::vmm::PageFlags;
use crate::{arch, elf_data, pmm, vmm};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use bitflags::Flags;
use core::arch::asm;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
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
    pub pid: u64,
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
    pub unsafe fn new(elf_data: &[u8], arg: usize) -> u64 {
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
            0x7FFF_FEFF_F000,
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

        let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);

        let process = Self {
            context,
            vm_root,
            state: ProcessState::Running,
            pending_signals: 0,
            exit_code: None,
            pid,
            sched_in: 0,
        };

        ALL.lock().insert(pid, process);
        println!("[cpu:{} created pid:{}]", arch::cpu_num(), pid);
        pid
    }

    pub fn run(id: u64) -> ! {
        println!("[cpu:{} running pid:{}]", arch::cpu_num(), id);
        let context = with(id, |p| {
            p.sched_in = PerCpu::ticks();
            PerCpu::set_running(Some(id));
            arch::load_tree(p.vm_root);
            p.context.clone()
        }).expect("Tried to load a process that doesn't exist");

        unsafe { arch::long_jump_context(&context) }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        arch::free_tree(self.vm_root);
        println!("[cpu:{} dropped pid:{}]", arch::cpu_num(), self.pid);
    }
}

static NEXT_PID: AtomicU64 = AtomicU64::new(1);
pub static ALL: Mutex<BTreeMap<u64, Process>> = Mutex::new(BTreeMap::new());
pub static RUNNABLE: Mutex<VecDeque<u64>> = Mutex::new(VecDeque::new());

pub fn schedule(proc: &Process) {
    schedule_pid(proc.pid);
}

pub fn schedule_pid(pid: u64) {
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

pub fn exit(code: u32) -> u64 {
    let Some(pid) = PerCpu::running() else {
        panic!("No running process");
    };
    ALL.lock().get_mut(&pid).unwrap().exit_code = Some(code);
    code as u64
}

pub fn spawn(_name: &str, arg: usize) -> u64 {
    unsafe {
        let pid = Process::new(&*elf_data(), arg);
        schedule_pid(pid);
        pid
    }
}

pub fn with<T, F: FnOnce(&mut Process) -> T>(pid: u64, func: F) -> Option<T> {
    ALL.lock().get_mut(&pid).map(func)
}