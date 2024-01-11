mod map;

use crate::arch::{Context, InterruptFrame, PageTable};
use crate::ipi::submit_ipi_to_all_cpus;
use crate::per_cpu::PerCpu;
use crate::println;
use crate::x86::print_backtrace_from_context;
use crate::{arch, elf_data};
use alloc::collections::{BTreeMap, VecDeque};
use core::cmp::min;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use cardinal3_interface::SyscallReturn;

pub struct Process {
    context: Context,
    vm_root: *mut PageTable,
    state: ProcessState,
    exit_code: Option<u64>,
    pid: u64,
    sched_in: u64,
    on_cpu: Option<usize>,
    tasks_to_wake: VecDeque<u64>,
    yield_context: Option<*mut [u64]>,
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
    pub unsafe fn new(elf_data: &'static [u8], arg: usize) -> u64 {
        let vm_root = arch::new_tree();
        let efile = map::map_elf_into_address_space(elf_data, vm_root);
        let mut context = Context::new_user(efile.ehdr.e_entry as usize);

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
            tasks_to_wake: VecDeque::new(),
            yield_context: None,
        };

        ALL.lock().insert(pid, process);
        pid
    }

    pub fn run(id: u64) -> ! {
        println!("[cpu:{} running pid:{}]", arch::cpu_num(), id);
        let context = with(id, |p| {
            PerCpu::set_running(Some(id));
            arch::load_tree(p.vm_root);
            p.on_cpu = Some(arch::cpu_num());
            p.sched_in = PerCpu::ticks();
            if p.state == ProcessState::Waiting {
                let Some(yield_context) = p.yield_context else {
                    panic!("waiting with nowhere to put tasks!");
                };

                let count = p.drain_tasks_to_wake(unsafe { &mut *yield_context });
                assert!(count > 0, "process woken up with nothing to do!");
                p.context.frame.set_syscall_return(SyscallReturn::Complete(0));
                p.context.frame.set_tasks_to_wake_count(count);
                p.yield_context = None;
                p.state = ProcessState::Running;
            }
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

    pub fn wait(&mut self, frame: &InterruptFrame) {
        self.yield_context = Some(frame.tasks_to_wake());
        self.state = ProcessState::Waiting;
    }

    pub fn unwait(&mut self) {
        self.yield_context = None;
        self.state = ProcessState::Running;
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

    pub fn drain_tasks_to_wake(&mut self, tasks: &mut [u64]) -> usize {
        let len = tasks.len();
        let count = min(len, self.tasks_to_wake.len());
        for (entry, task_id) in tasks.iter_mut().zip(self.tasks_to_wake.drain(..count)) {
            *entry = task_id;
        }

        count
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

pub fn with<T, F: FnMut(&mut Process) -> T>(pid: u64, func: F) -> Option<T> {
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
    submit_ipi_to_all_cpus(|| backtrace_local());
}

pub fn schedule_wakeup(pid: u64, task_id: u64) {
    with(pid, |proc| {
        proc.tasks_to_wake.push_back(task_id);
        schedule_pid(pid);
    });
}