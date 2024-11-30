// Much of this module models real-world hardware bits. Some of those bits aren't currently
// used by this system.
#![allow(dead_code)]

use alloc::alloc::Allocator;
use acpi::platform::interrupt::InterruptSourceOverride;

const IOAPIC_BASE: usize = 0xFEC0_0000;
const MAPPED_BASE: usize = 0xFFFF_8000_0000_0000 + IOAPIC_BASE;
const MAPPED_ADDR: *mut u32 = MAPPED_BASE as *mut u32;
const MAPPED_DATA: *mut u32 = (MAPPED_BASE + 0x10) as *mut u32;

pub unsafe fn write(offset: usize, value: u32) {
    MAPPED_ADDR.write_volatile(offset as u32);
    MAPPED_DATA.write_volatile(value);
}

pub unsafe fn read(offset: usize) -> u32 {
    MAPPED_ADDR.write_volatile(offset as u32);
    MAPPED_DATA.read_volatile()
}

enum DeliveryMode {
    Fixed = 0b000,
    LowPriority = 0b001,
    Smi = 0b010,
    Nmi = 0b100,
    Init = 0b101,
    ExtInt = 0b111,
}

enum DestinationMode {
    Physical = 0,
    Logical = 1,
}

enum DeliveryStatus {
    Idle = 0,
    SendPending = 1,
}

#[allow(dead_code)] // modelling real-world
enum PinPolarity {
    High = 0,
    Low = 1,
}

impl From<&acpi::platform::interrupt::Polarity> for PinPolarity {
    fn from(polarity: &acpi::platform::interrupt::Polarity) -> Self {
        match polarity {
            acpi::platform::interrupt::Polarity::ActiveHigh => Self::High,
            acpi::platform::interrupt::Polarity::SameAsBus => Self::High,
            acpi::platform::interrupt::Polarity::ActiveLow => Self::Low,
        }
    }
}

enum RemoteIRR {
    Disabled = 0,
    Enabled = 1,
}

enum TriggerMode {
    Edge = 0,
    Level = 1,
}

impl From<&acpi::platform::interrupt::TriggerMode> for TriggerMode {
    fn from(trigger_mode: &acpi::platform::interrupt::TriggerMode) -> Self {
        match trigger_mode {
            acpi::platform::interrupt::TriggerMode::Edge => Self::Edge,
            acpi::platform::interrupt::TriggerMode::SameAsBus => Self::Edge,
            acpi::platform::interrupt::TriggerMode::Level => Self::Level,
        }
    }
}

enum Mask {
    Allowed = 0,
    Deferred = 1,
}

struct RelocationEntry {
    vector: u8,
    destination_apic: u8,
    delivery_mode: DeliveryMode,
    destination_mode: DestinationMode,
    delivery_status: DeliveryStatus,
    pin_polarity: PinPolarity,
    remote_irr: RemoteIRR,
    trigger_mode: TriggerMode,
    mask: Mask,
}

impl RelocationEntry {
    fn new(vector: u8, destination: u8) -> Self {
        Self {
            vector,
            destination_apic: destination,
            delivery_mode: DeliveryMode::Fixed,
            destination_mode: DestinationMode::Physical,
            delivery_status: DeliveryStatus::Idle,
            pin_polarity: PinPolarity::High,
            remote_irr: RemoteIRR::Disabled,
            trigger_mode: TriggerMode::Edge,
            mask: Mask::Deferred,
        }
    }

    fn into_bits(self) -> (u32, u32) {
        let full = (self.vector as u64)
            | (self.delivery_mode as u64) << 8
            | (self.destination_mode as u64) << 11
            | (self.delivery_status as u64) << 12
            | (self.pin_polarity as u64) << 13
            | (self.remote_irr as u64) << 14
            | (self.trigger_mode as u64) << 15
            | (self.mask as u64) << 16
            | (self.destination_apic as u64) << 56;
        ((full >> 32) as u32, full as u32)
    }
}

impl From<&InterruptSourceOverride> for RelocationEntry {
    fn from(value: &InterruptSourceOverride) -> Self {
        Self {
            vector: value.isa_source + 32,
            destination_apic: 0,
            delivery_mode: DeliveryMode::Fixed,
            destination_mode: DestinationMode::Physical,
            delivery_status: DeliveryStatus::Idle,
            pin_polarity: (&value.polarity).into(),
            remote_irr: RemoteIRR::Disabled,
            trigger_mode: (&value.trigger_mode).into(),
            mask: Mask::Deferred,
        }
    }
}

pub unsafe fn init<A: Allocator>(interrupt_info: &acpi::platform::interrupt::Apic<A>) {
    for gsi in 0..16 {
        let entry = RelocationEntry::new(gsi + 32, 0);
        let (high, low) = entry.into_bits();

        let offset = 0x10 + (gsi as usize) * 2;

        write(offset, low);
        write(offset + 1, high);
    }

    for iso in interrupt_info.interrupt_source_overrides.iter() {
        let gsi = iso.global_system_interrupt;

        let entry = RelocationEntry::from(iso);
        let (high, low) = entry.into_bits();

        let offset = 0x10 + (gsi as usize) * 2;

        write(offset, low);
        write(offset + 1, high);
    }
}

pub unsafe fn mask_irq(gsi: u8) {
    let offset = 0x10 + (gsi as usize) * 2;
    let low = read(offset);
    let high = read(offset + 1);
    write(offset, low | (1 << 16));
    write(offset + 1, high);
}

pub unsafe fn unmask_irq(gsi: u8) {
    let offset = 0x10 + (gsi as usize) * 2;
    let low = read(offset);
    let high = read(offset + 1);
    write(offset, low & !(1 << 16));
    write(offset + 1, high);
}
