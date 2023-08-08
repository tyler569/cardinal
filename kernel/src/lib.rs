#![no_std]
#![allow(unused)]
#![feature(naked_functions)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(pointer_byte_offsets)]
#![feature(noop_waker)]

extern crate alloc;

use core::arch::asm;
use core::pin::pin;
use core::time::Duration;

mod allocator;
mod async_test;
mod executor;
mod limine;
mod mem;
mod per_cpu;
mod print;
mod thread;
mod timer;
mod x86;

use crate::arch::SERIAL;
use crate::per_cpu::PerCpu;
pub(crate) use print::{print, println};
pub(crate) use x86 as arch;

pub const NUM_CPUS: usize = 16;

#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    mem::static_heap_init();
    PerCpu::init();
    arch::early_system_init();
    arch::long_jump(kernel_main as usize)
}

unsafe extern "C" fn kernel_main() -> ! {
    asm!("int3");

    // limine_info();

    start_ap();

    arch::enable_interrupts();

    // timer::insert(Duration::from_secs(1), || println!("timer 1"));
    // timer::insert(Duration::from_secs(1), || arch::send_ipi(1, 129));

    // let res = async_test::run_async(async_test::foobar(10, 11));
    // println!("async result: {}", res);

    println!("spawning sleep task");
    executor::spawn(async {
        loop {
            executor::sleep::sleep(Duration::from_millis(300)).await;
            print!(".")
        }
    });

    println!("spawning serial task");
    executor::spawn(async {
        loop {
            let c = SERIAL.read();
            let c = c.await;
            print!("{}", c as char);
        }
    });

    executor::spawn(async {
        loop {
            executor::sleep::sleep(Duration::from_millis(3000)).await;
            panic!();
        }
    });

    loop {
        PerCpu::get_mut().executor.do_work();
        asm!("hlt");
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

    arch::long_jump(ap_main as usize)
}

unsafe fn ap_main() -> ! {
    arch::enable_interrupts();
    arch::sleep_forever()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    arch::broadcast_ipi(130);

    println!("PANIC: {}", info);

    arch::sleep_forever_no_irq()
}

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

unsafe fn start_ap() {
    let smp = &**limine::SMP.response.get();
    let Some(cpu) = smp.cpus_slice().get(1) else {
        return;
    };

    (**cpu).goto_address = ap_init;
}
