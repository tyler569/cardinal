use crate::arch::{Context, InterruptFrame, PageTable};
use crate::per_cpu::PerCpu;
use crate::vmm::PageFlags;
use crate::{arch, elf_data, pmm};
use crate::println;
use alloc::collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicU64, Ordering};
use elf::endian::LittleEndian;
use spin::Mutex;
use crate::ipi::submit_ipi_to_all_cpus;
use crate::x86::print_backtrace_from_context;

#[derive(Debug)]
pub struct Process {
    context: Context,
    vm_root: *mut PageTable,
    state: ProcessState,
    exit_code: Option<u64>,
    pid: u64,
    sched_in: u64,
    on_cpu: Option<usize>,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessDisposition {
    MayContinue,
    NotNow,
    TimesUp,
    NeverAgain,
}

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

                    for i in 0..(ph.p_memsz as usize + 0xfff) / 0x1000 {
                        let offset = i * 0x1000;
                        let page_vma = base + ph.p_offset as usize + offset;
                        let page_phy = pmm::alloc().unwrap();
                        let hhdm_vma = arch::direct_map_offset(page_phy);
                        let mapped_vma = ph.p_vaddr as usize + offset;

                        let copy_len = core::cmp::min(0x1000, (ph.p_filesz as usize).saturating_sub(offset));
                        let zero_len = 0x1000 - copy_len;

                        if copy_len > 0 {
                            core::ptr::copy_nonoverlapping(page_vma as *const u8, hhdm_vma as *mut u8, copy_len);
                        }
                        if zero_len > 0 {
                            core::ptr::write_bytes((hhdm_vma + copy_len) as *mut u8, 0, zero_len);
                        }

                        arch::map_in_table(vm_root, mapped_vma, page_phy, flags);
                    }
                } else {
                    if ph.p_flags & elf::abi::PF_X != 0 {
                        flags |= PageFlags::EXECUTE;
                    }
                    for i in 0..(ph.p_memsz as usize + 0xfff) / 0x1000 {
                        let page_vma = base + ph.p_offset as usize + i * 0x1000;
                        let page_phy = arch::physical_address(page_vma).unwrap() & !0xfff;
                        let mapped_vma = (ph.p_vaddr as usize + i * 0x1000) & !0xfff;

                        arch::map_in_table(vm_root, mapped_vma, page_phy, flags);
                    }
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
            exit_code: None,
            pid,
            sched_in: 0,
            on_cpu: None,
        };

        ALL.lock().insert(pid, process);
        // println!("[cpu:{} created pid:{}]", arch::cpu_num(), pid);
        pid
    }

    pub fn run(id: u64) -> ! {
        println!("[cpu:{} running pid:{}]", arch::cpu_num(), id);
        let context = with(id, |p| {
            PerCpu::set_running(Some(id));
            arch::load_tree(p.vm_root);
            p.on_cpu = Some(arch::cpu_num());
            p.sched_in = PerCpu::ticks();
            p.context.clone()
        })
        .expect("Tried to load a process that doesn't exist");

        unsafe { arch::long_jump_context(&context) }
    }

    pub fn time_expired(&self) -> bool {
        self.on_cpu.is_some() && self.sched_in + 10 > PerCpu::ticks()
    }

    pub fn should_run(&self) -> ProcessDisposition {
        match self.state {
            ProcessState::Exited => ProcessDisposition::NeverAgain,
            ProcessState::Waiting => ProcessDisposition::NotNow,
            ProcessState::Running => {
                if self.time_expired() {
                    ProcessDisposition::TimesUp
                } else {
                    ProcessDisposition::MayContinue
                }
            }
        }
    }

    pub fn exit(&mut self, code: u64) {
        self.state = ProcessState::Exited;
        self.exit_code = Some(code);
    }

    #[allow(dead_code)]
    pub fn wait(&mut self, _task_id: u64) {
        self.state = ProcessState::Waiting;
    }

    pub fn vm_root(&self) -> *mut PageTable {
        self.vm_root
    }

    pub fn set_context(&mut self, frame: &InterruptFrame) {
        self.context = Context::new(frame);
    }

    pub fn set_on_cpu(&mut self, on_cpu: Option<usize>) {
        self.on_cpu = on_cpu;
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if RUNNABLE.lock().iter().any(|&pid| pid == self.pid) {
            panic!("dropping process that exists on the runnable queue");
        }
        arch::free_tree(self.vm_root);
        println!("[cpu:{} dropped pid:{}]", arch::cpu_num(), self.pid);
    }
}

static NEXT_PID: AtomicU64 = AtomicU64::new(1);
pub static ALL: Mutex<BTreeMap<u64, Process>> = Mutex::new(BTreeMap::new());
pub static RUNNABLE: Mutex<VecDeque<u64>> = Mutex::new(VecDeque::new());

pub fn schedule_pid(pid: u64) {
    let mut handle = RUNNABLE.lock();
    // assert!(handle.iter().all(|&p| p != pid), "double scheduling {}!", pid);
    if handle.iter().any(|&p| p == pid) {
        return;
    }
    // println!("[cpu:{} scheduling pid:{}]", arch::cpu_num(), pid);
    handle.push_back(pid);
}

pub fn maybe_run_usermode_program(swap_in_current: bool) {
    let pid = {
        let mut binding = RUNNABLE.lock();
        let Some(pid) = binding.pop_front() else {
            return;
        };
        if swap_in_current {
            binding.push_back(PerCpu::running().unwrap())
        }
        pid
    };
    Process::run(pid)
}

pub fn exit(code: u64) -> u64 {
    let Some(pid) = PerCpu::running() else {
        panic!("No running process");
    };
    with(pid, |p| p.exit(code));
    code
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

pub fn remove(pid: u64) {
    ALL.lock().remove(&pid);
}

pub fn backtrace_local() {
    let Some(pid) = PerCpu::running() else {
        return;
    };

    with(pid, |p| print_backtrace_from_context(&p.context));
}

pub fn backtrace_all() {
    submit_ipi_to_all_cpus(|| { backtrace_local() });
}