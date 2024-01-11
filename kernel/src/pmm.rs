use crate::limine::mmap::{LimineMmapEntryType};
use crate::print::println;
use crate::limine;
use alloc::vec::Vec;
use spin::Mutex;

#[derive(Debug, Copy, Clone)]
enum PageInfo {
    NoMemory,
    Free,
    Reserved,
    InUse { refcount: u16 },
    Kernel,

    // We don't support refcounting yet, so there's no code for leaking pages.
    #[allow(dead_code)]
    Leaked,
}

static PAGE_INFO: Mutex<Vec<PageInfo>> = Mutex::new(Vec::new());

pub fn init() {
    let mut page_info = PAGE_INFO.lock();

    let limine_mmap = unsafe { &**limine::MMAP.response.get() };
    let mut page_count = 0;
    for entry in limine_mmap.entries_slice() {
        let entry = unsafe { &**entry };
        // println!("{:x?}", entry);
        match entry.typ {
            LimineMmapEntryType::BootloaderReclaimable
            | LimineMmapEntryType::Usable
            | LimineMmapEntryType::KernelAndModules => {
                page_count = core::cmp::max(page_count, (entry.base + entry.len) / 4096);
            }
            _ => {}
        }
    }

    println!("PageInfo size: {}", core::mem::size_of::<PageInfo>());
    println!(
        "reserving space for {} pages ({} KiB) (top {:x})",
        page_count,
        page_count * 4,
        page_count * 4096
    );

    page_info.resize(page_count as usize, PageInfo::NoMemory);

    for entry in limine_mmap.entries_slice() {
        let entry = unsafe { &**entry };
        let start_page = entry.base / 4096;
        let end_page = (entry.base + entry.len) / 4096;

        let mut fill_with = |typ| {
            for page in start_page..end_page {
                if page >= page_count {
                    break;
                }
                page_info[page as usize] = typ;
            }
        };

        match entry.typ {
            LimineMmapEntryType::Usable => fill_with(PageInfo::Free),
            LimineMmapEntryType::KernelAndModules => fill_with(PageInfo::Kernel),

            LimineMmapEntryType::BootloaderReclaimable => fill_with(PageInfo::Reserved),
            LimineMmapEntryType::Reserved => fill_with(PageInfo::Reserved),
            LimineMmapEntryType::Framebuffer => fill_with(PageInfo::Reserved),
            LimineMmapEntryType::AcpiReclaimable => fill_with(PageInfo::Reserved),
            LimineMmapEntryType::AcpiNvs => fill_with(PageInfo::Reserved),
            LimineMmapEntryType::BadMemory => fill_with(PageInfo::Reserved),
        }
    }
}

pub fn alloc() -> Option<u64> {
    let mut page_info = PAGE_INFO.lock();
    for (i, page) in page_info.iter_mut().enumerate() {
        if let PageInfo::Free = page {
            *page = PageInfo::InUse { refcount: 1 };
            return Some((i * 4096) as u64);
        }
    }
    None
}

pub fn alloc_contiguous(pages: usize) -> Option<u64> {
    let mut page_info = PAGE_INFO.lock();
    let mut start = 0;
    let mut count = 0;
    for (i, page) in page_info.iter_mut().enumerate() {
        match page {
            PageInfo::Free => {
                if count == 0 {
                    start = i;
                }
                count += 1;
                if count == pages {
                    for page in &mut page_info[start..start + pages] {
                        *page = PageInfo::InUse { refcount: 1 };
                    }
                    return Some((start * 4096) as u64);
                }
            }
            _ => {
                count = 0;
            }
        }
    }
    None
}

pub fn free(page: u64) {
    let mut page_info = PAGE_INFO.lock();
    let page = (page / 4096) as usize;
    if let PageInfo::InUse { refcount } = page_info[page] {
        assert!(refcount > 0);
    }
    page_info[page] = match page_info[page] {
        PageInfo::InUse { refcount: 1 } => PageInfo::Free,
        PageInfo::InUse { refcount } => PageInfo::InUse { refcount: refcount - 1 },
        other => other,
    }
}

pub fn summary() {
    let page_info = PAGE_INFO.lock();
    let mut free = 0;
    let mut reserved = 0;
    let mut kernel = 0;
    let mut leaked = 0;
    for page in &*page_info {
        match page {
            PageInfo::Free => free += 1,
            PageInfo::Reserved => reserved += 1,
            PageInfo::Kernel => kernel += 1,
            PageInfo::Leaked => leaked += 1,
            _ => {}
        }
    }
    println!(
        "free: {}, reserved: {}, kernel: {}, leaked: {}",
        free, reserved, kernel, leaked
    );
}
