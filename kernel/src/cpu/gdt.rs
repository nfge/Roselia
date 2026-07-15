use lazy_static::lazy_static;
use x86_64::{VirtAddr, registers::segmentation::{CS, SS, Segment}, structures::{
    gdt::{Descriptor, GlobalDescriptorTable},
    tss::TaskStateSegment,
}};

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
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE as u64
        };
        tss
    };
}

pub fn init() {
    GDT.load();
}
