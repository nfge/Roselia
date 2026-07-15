use lazy_static::lazy_static;
use x86_64::{VirtAddr, instructions::tables::load_tss, registers::segmentation::{CS,Segment}, structures::{
    gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    tss::TaskStateSegment,
}};

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, SegmentSelector, SegmentSelector) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_sel = gdt.append(Descriptor::kernel_code_segment());
        gdt.append(Descriptor::kernel_data_segment());
        let tss_sel = gdt.append(Descriptor::tss_segment(&TSS));
        (gdt, code_sel,tss_sel)
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
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1);
        load_tss(GDT.2);
    }
}
