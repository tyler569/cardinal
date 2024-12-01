#![no_std]
#![no_main]

#![feature(allocator_api)]
#![feature(const_trait_impl)]
#![feature(int_roundings)]
#![feature(naked_functions)]
#![feature(slice_ptr_get)]

#![allow(unused)]

extern crate alloc;

use cardinal3_allocator as allocator;
use core::arch::asm;
use core::time::Duration;

mod executor;
mod ipi;
mod limine;
mod mem;
mod net;
mod pci;
mod per_cpu;
mod pmm;
mod print;
mod process;
mod syscalls;
mod timer;
mod vmm;
mod x86;

use crate::arch::SERIAL;
use crate::per_cpu::PerCpu;
use crate::process::Process;
use print::{print, println};
use x86 as arch;

pub const NUM_CPUS: usize = 16;
pub const START_PROCS: usize = 1;

#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    mem::static_heap_init();
    PerCpu::init();
    arch::early_system_init();
    pmm::init();
    arch::long_jump_cs(kernel_main as usize)
}

unsafe extern "C" fn kernel_main() -> ! {
    asm!("int3");

    // limine_info();

    start_aps();

    // println!("spawning int3 task");
    // executor::spawn(async { asm!("int3"); });

    println!("spawning sleep task");
    executor::spawn(async {
        loop {
            executor::sleep::sleep(Duration::from_secs(1)).await;
            print!(".")
        }
    });

    println!("spawning serial task");
    executor::spawn(async {
        loop {
            let c = SERIAL.read().await;
            match c {
                b's' => load_and_start_usermode_program(0),
                b'm' => pmm::summary(),
                b'p' => process::backtrace_all(),
                b'b' => arch::breakpoint(),
                b'B' => executor::spawn(async { arch::breakpoint() }),
                _ => {}
            }
            print!("{}", c as char);
        }
    });

    // pci::enumerate_pci_bus();
    // if let Some(rtl_addr) = pci::find_device(0x10ec, 0x8139) {
    //     println!("found rtl8139 at {}", rtl_addr);
    //     let mut rtl_device = pci::rtl8139::Rtl8139::new(rtl_addr);
    //     rtl_device.init();

    //     rtl_device.send_packet(&[
    //         0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // dest
    //         0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // source
    //         0x00, 0x10, // size
    //         0x01, 0x02, // payload
    //     ]);
    // }

    for _ in 0..START_PROCS {
        load_and_start_usermode_program(0);
    }

    arch::sleep_forever()
}

unsafe extern "C" fn ap_init(info: *const limine::smp::LimineCpuInfo) -> ! {
    arch::early_cpu_init();

    println!(
        "ap_init (number {}, cpu {})",
        unsafe { (*info).processor_id },
        arch::cpu_num()
    );

    arch::long_jump_cs(ap_main as usize)
}

unsafe fn ap_main() -> ! {
    arch::sleep_forever();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    arch::broadcast_ipi(130);

    println!("PANIC: {}", info);

    arch::sleep_forever_no_irq()
}

#[allow(unused)]
unsafe fn limine_info() {
    let boot_info = &**limine::BOOT_INFO.response.get();
    println!("boot_info: {:?}", boot_info);

    let hhdm = &**limine::HHDM.response.get();
    println!("hhdm info: {:x?}", hhdm);

    let smp = &**limine::SMP.response.get();
    println!("smp info : {:x?}", smp);
    for (i, cpu) in smp.cpus_slice().iter().enumerate() {
        println!("cpu[{}]: {:?}", i, **cpu);
    }

    let mmap = &**limine::MMAP.response.get();
    println!("mmap info: {:?}", mmap);
    for (i, entry) in mmap.entries_slice().iter().enumerate() {
        println!("mmap[{}]: {:x?}", i, **entry);
    }
}

unsafe fn start_aps() {
    let smp = &**limine::SMP.response.get();

    for cpu in smp.cpus_slice().iter().skip(1) {
        *(**cpu).goto_address.get_mut() = ap_init;
    }
}

fn elf_data() -> *const [u8] {
    unsafe {
        let mods_info = &**limine::MODULE.response.get();
        let mod_info = &*mods_info.modules_slice()[0];
        mod_info.data()
    }
}

unsafe fn load_and_start_usermode_program(arg: usize) {
    process::schedule_pid(Process::new(&*elf_data(), arg));
}
