use crate::{limine, x86};
use crate::print::println;
use acpi::{AcpiTables, PhysicalMapping};
use core::ptr::NonNull;

#[derive(Copy, Clone, Debug)]
pub struct AcpiHandler;

impl acpi::AcpiHandler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let offset = x86::direct_map_offset();

        let virtual_pointer;
        if physical_address < offset {
            let virtual_address = physical_address + offset;
            virtual_pointer = NonNull::new(virtual_address as *mut T).unwrap();
        } else {
            virtual_pointer = NonNull::new(physical_address as *mut T).unwrap();
        }

        return PhysicalMapping::new(physical_address, virtual_pointer, size, size, AcpiHandler);
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {}
}

pub unsafe fn init() -> AcpiTables<AcpiHandler> {
    let rsdp_address = unsafe { (**limine::RSDP.response.get()).address } as usize;
    AcpiTables::from_rsdp(AcpiHandler, rsdp_address).unwrap()
}
