use core::marker::PhantomData;

use crate::acpi_tables::sdtheader::SdtHeader;

#[repr(C, packed)]
pub struct Madt {
    pub header: SdtHeader,
    pub local_apic_address: u32,
    pub flags: u32,
}

#[repr(C, packed)]
struct MadtEntryHeader {
    entry_type: u8,
    length: u8,
}

#[repr(C, packed)]
pub struct LocalApic {
    pub acpi_processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

#[repr(C, packed)]
pub struct IoApic {
    pub io_apic_id: u8,
    reserved: u8,
    pub io_apic_address: u32,
    pub global_system_interrupt_base: u32,
}
#[repr(C, packed)]
pub struct InterruptSourceOverride {
    pub bus: u8,
    pub source: u8,
    pub global_system_interrupt: u32,
    pub flags: u16,
}

#[repr(C, packed)]
pub struct LocalApicNmi {
    pub acpi_processor_id: u8,
    pub flags: u16,
    pub lint_number: u8,
}

#[repr(C, packed)]
pub struct LocalX2Apic {
    reserved: u16,
    pub x2apic_id: u32,
    pub flags: u32,
    pub acpi_processor_uid: u32,
}

pub enum MadtEntry<'a> {
    LocalApic(&'a LocalApic),
    IoApic(&'a IoApic),
    InterruptSourceOverride(&'a InterruptSourceOverride),
    LocalX2Apic(&'a LocalX2Apic),
    Unknown(u8),
}

impl Madt {
    pub unsafe fn entries(&self) -> MadtEntryIter<'_> {
        let start = unsafe {(self as *const Madt as *const u8).add(core::mem::size_of::<Madt>())};
        let end = unsafe{(self as *const Madt as *const u8).add(self.header.length as usize)};
        MadtEntryIter { ptr: start, end, _marker: PhantomData }
    }
}

pub struct MadtEntryIter<'a> {
    ptr: *const u8,
    end: *const u8,
    _marker: PhantomData<&'a Madt>
}

impl<'a> Iterator for MadtEntryIter<'a> {
    type Item = MadtEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.end {
            return None;
        }
        unsafe {
            let entry_type = *self.ptr;
            let length = *self.ptr.add(1) as usize;
            let body_ptr = self.ptr.add(2);

            let entry = match entry_type {
                0 => MadtEntry::LocalApic(&*(body_ptr as *const LocalApic)),
                1 => MadtEntry::IoApic(&*(body_ptr as *const IoApic)),
                2 => MadtEntry::InterruptSourceOverride(
                    &*(body_ptr as *const InterruptSourceOverride),
                ),
                9 => MadtEntry::LocalX2Apic(&*(body_ptr as *const LocalX2Apic)),
                other => MadtEntry::Unknown(other),
            };

            self.ptr = self.ptr.add(length);
            Some(entry)
        }
    }
}
