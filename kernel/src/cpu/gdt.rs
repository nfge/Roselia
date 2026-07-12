use lazy_static::lazy_static;
use x86_64::structures::{gdt::{Descriptor, GlobalDescriptorTable}, tss::TaskStateSegment};

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();
        gdt.append(Descriptor::kernel_code_segment());
        gdt.append(Descriptor::kernel_data_segment());
        gdt.append(Descriptor::tss_segment(&TSS));
        gdt
    };
}
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let tss = TaskStateSegment::new();
        tss
    };
}

pub fn init() {
    GDT.load();
}