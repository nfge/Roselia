use core::alloc::Layout;
use core::ffi::c_void;
use core::fmt::Arguments;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::acpi_tables::mcfg::McfgEntry;
use crate::logger::loglevel::LogLevel;
use crate::pci::PciDevice;

#[cfg(feature = "alloc")]
use crate::ramfs::error::RamFSError;

use crate::thread::ThreadId;

unsafe extern "Rust" {
    pub fn kprint(s: &str);
    pub fn kprintln(s: &str);
    pub fn kprintf(arg: core::fmt::Arguments);
    pub fn kprintfln(arg: core::fmt::Arguments);
    pub fn kalloc(layout: Layout) -> Result<core::ptr::NonNull<u8>, ()>;
    pub fn kfree(ptr: core::ptr::NonNull<u8>, layout: Layout);

    #[cfg(feature = "alloc")]
    pub fn read(path: &str) -> Result<Vec<u8>, RamFSError>;

    #[cfg(feature = "alloc")]
    pub fn write(path: &str, offset: usize, data: &[u8]) -> Result<(), RamFSError>;

    #[cfg(feature = "alloc")]
    pub fn spawn_box(f: Box<dyn FnOnce() -> i32 + Send + 'static>) -> ThreadId;

    #[cfg(feature = "alloc")]
    pub fn spawnc_box(f: Box<dyn FnOnce() -> i32 + Send + 'static>) -> ThreadId;

    pub fn wait(tid: ThreadId) -> Option<i32>;
    pub fn kill(tid: ThreadId);
    pub fn thread_sleep(time_ms: u64);

    pub fn klog(args: Arguments, level: LogLevel);
    pub fn get_acpi_table() -> *const c_void;
    pub fn get_ptr_table(
        acpi_table: *const c_void,
        signature: &[u8; 4],
    ) -> Option<*const c_void>;
    pub fn pci_check(vendor_id: u16, device_id: u16) -> (Option<&'static str>, Option<&'static str>);
    pub fn pci_find_by_id(mcfg_entry: &McfgEntry, vendor_id: u16, device_id: u16) -> Option<PciDevice>;
    pub fn pci_find_by_class(mcfg_entry: &McfgEntry, class: u8, subclass: u8) -> Option<PciDevice>;
    pub fn module_panic() -> !;
}
#[cfg(feature = "alloc")]
pub fn spawn<F: FnOnce() -> i32 + Send + 'static>(f: F) -> ThreadId {
    unsafe {spawn_box(Box::new(f))}
}
#[cfg(feature = "alloc")]
pub fn spawnc<F: FnOnce() -> i32 + Send + 'static>(f: F) -> ThreadId {
    unsafe {spawnc_box(Box::new(f))}
}