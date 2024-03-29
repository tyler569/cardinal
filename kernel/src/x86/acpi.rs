use crate::limine;
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
        let virtual_pointer = NonNull::new(physical_address as *mut T).unwrap();

        PhysicalMapping::new(physical_address, virtual_pointer, size, size, AcpiHandler)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

pub unsafe fn init() -> AcpiTables<AcpiHandler> {
    let rsdp_address = unsafe { (**limine::RSDP.response.get()).address } as usize;
    AcpiTables::from_rsdp(AcpiHandler, rsdp_address).unwrap()
}
