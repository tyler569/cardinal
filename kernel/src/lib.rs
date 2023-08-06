#![no_std]
#![allow(unused)]
#![feature(naked_functions)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(pointer_byte_offsets)]

use core::arch::asm;

mod alloc;
mod limine;
mod mem;
mod print;
mod thread;
mod x86;

pub(crate) use print::{print, println};
pub(crate) use x86 as arch;

#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    mem::static_heap_init();
    arch::early_system_init();
    println!("Hello, world!");
    arch::long_jump(kernel_main as usize)
}

unsafe extern "C" fn kernel_main() -> ! {
    println!("kernel_main");
    asm!("int3");

    // limine_info();

    println!("this is cpu number {}", arch::cpu_num());
    start_ap();

    arch::enable_interrupts();

    arch::sleep(core::time::Duration::from_secs(1));
    println!("sending IPI to cpu 1");
    arch::send_ipi(1, 129);

    arch::sleep_forever()
}

extern "C" fn ap_init(info: *const limine::smp::LimineCpuInfo) -> ! {
    unsafe { arch::early_cpu_init() };

    println!("this is the ap! (number {})", unsafe {
        (*info).processor_id
    });
    println!("this is cpu number {}", arch::cpu_num());

    unsafe { arch::long_jump(ap_main as usize) }
}

unsafe fn ap_main() -> ! {
    asm!("int3");

    arch::enable_interrupts();
    arch::sleep_forever()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
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
        return
    };

    (**cpu).goto_address = ap_init;
}