use core::{ffi::c_void, panic::PanicInfo};

use acpi::{get_ptr_table, get_table};
use pci::{find_by_class, find_by_id};

use crate::{ACPI_TABLE, cpu::cpuinfo::get_cpu, export_symbol, kprintln, memory::{kalloc, kfree}, ramfs::read_file, terminal::export::{kprint, kprintln}};


pub fn init_exports() {
    export_symbol!("kprint", kprint);
    export_symbol!("kprintln", kprintln);
    export_symbol!("kalloc", kalloc);
    export_symbol!("kfree", kfree);
    export_symbol!("read", read_file);
    export_symbol!("get_acpi_table", get_acpi_table);
    export_symbol!("get_ptr_table", get_ptr_table);
    export_symbol!("pci_find_by_id", find_by_id);
    export_symbol!("pci_find_by_class", find_by_class);
    export_symbol!("get_cpu", get_cpu);
}

pub extern "Rust" fn get_acpi_table() -> *const c_void {
    unsafe { ACPI_TABLE.unwrap() }
}

fn module_panic(info: &PanicInfo) {
    kprintln!("{}", info);
}
