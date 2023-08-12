#![no_std]
#![allow(unused)]
#![feature(naked_functions)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(pointer_byte_offsets)]

extern crate alloc;

use core::arch::asm;
use core::pin::pin;
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use elf::endian::LittleEndian;

mod allocator;
mod executor;
mod limine;
mod mem;
mod net;
mod pci;
mod per_cpu;
mod pmm;
mod print;
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
    pmm::init();
    arch::long_jump(kernel_main as usize)
}

unsafe extern "C" fn kernel_main() -> ! {
    asm!("int3");

    // limine_info();

    start_aps();

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
            print!("{}", SERIAL.read().await as char);
        }
    });

    // println!("spawning panic task");
    // executor::spawn(async {
    //     loop {
    //         executor::sleep::sleep(Duration::from_secs(3)).await;
    //         panic!();
    //     }
    // });

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

    load_and_start_usermode_program();

    executor::work_forever()
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
    executor::work_forever()
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

unsafe fn start_aps() {
    let smp = &**limine::SMP.response.get();

    for cpu in smp.cpus_slice().iter().skip(1) {
        (**cpu).goto_address = ap_init;
    }
}

unsafe fn load_and_start_usermode_program() {
    let mods_info = &**limine::MODULE.response.get();
    let mod_info = &*mods_info.modules_slice()[0];
    let mod_data = mod_info.data();
    println!("mod data at {:?}", mod_data);

    let efile = elf::ElfBytes::<LittleEndian>::minimal_parse(&*mod_data).unwrap();
    let segments = efile.segments().unwrap();
    for header in segments {
        if header.p_type == elf::abi::PT_LOAD {
            println!("load offset {:#x} size {:#x} to {:#x}", header.p_offset, header.p_filesz, header.p_vaddr);
            let page_vma = (header.p_vaddr & !0xfff) as usize;
            let page_phy = arch::physical_address((mod_data as *const u8 as usize + header.p_offset as usize) & !0xfff).unwrap();
            arch::map(page_vma, page_phy, arch::Pte::WRITEABLE | arch::Pte::USERMODE);
        }
    }

    const STACK: usize = 0x7FFF_FF00_0000;
    arch::map(STACK, pmm::alloc().unwrap(), arch::Pte::WRITEABLE | arch::Pte::USERMODE);
    println!("jump to entrypoint {:#x}", efile.ehdr.e_entry);
    // arch::sleep_forever_no_irq();

    arch::long_jump_usermode(efile.ehdr.e_entry as usize, STACK + 0xFF0);
}