#![no_std]
#![allow(unused)]
#![feature(naked_functions)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(pointer_byte_offsets)]

use core::arch::asm;

mod alloc;
mod arch;
mod limine;
mod mem;
mod print;
mod thread;
mod x86;

pub(crate) use arch::Arch;
pub(crate) use print::{print, println};
pub(crate) use x86::X86ARCH as ARCH;

#[no_mangle]
pub unsafe extern "C" fn kernel_init() -> ! {
    mem::static_heap_init();
    ARCH.early_system_init();
    println!("Hello, world!");
    ARCH.long_jump(kernel_main as usize)
}

unsafe extern "C" fn kernel_main() -> ! {
    println!("kernel_main");
    asm!("int3");

    // limine_info();

    println!("this is cpu number {}", ARCH.cpu_num());
    start_ap();

    ARCH.enable_interrupts();

    ARCH.sleep(core::time::Duration::from_secs(1));
    println!("sending IPI to cpu 1");
    ARCH.send_ipi(1, 129);

    ARCH.sleep_forever()
}

extern "C" fn ap_init(info: *const limine::smp::LimineCpuInfo) -> ! {
    unsafe { ARCH.early_cpu_init() };

    println!("this is the ap! (number {})", unsafe {
        (*info).processor_id
    });
    println!("this is cpu number {}", ARCH.cpu_num());

    unsafe { ARCH.long_jump(ap_main as usize) }
}

unsafe fn ap_main() -> ! {
    asm!("int3");

    ARCH.enable_interrupts();
    ARCH.sleep_forever()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info);

    ARCH.sleep_forever_no_irq()
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