use core::{ops::Add, sync::atomic::AtomicU64};

use crate::ramfs::{is_valid, write_file};

pub struct Logger ;

pub static LOGGER_OFFSET: AtomicU64 = AtomicU64::new(0);

impl Logger {
    pub fn write(line: &str) {
        let data: &[u8] = line.as_bytes(); 
        let _ = is_valid("/kernel/log").unwrap();
        let _ = write_file("/kernel/log", LOGGER_OFFSET.load(core::sync::atomic::Ordering::Relaxed) as usize, data).unwrap();
        LOGGER_OFFSET.fetch_add(data.len() as u64, core::sync::atomic::Ordering::Relaxed);
    }
}

impl core::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Self::write(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut logger = crate::logger::Logger;
        let _ = write!(logger, $($arg)*);
    }};
}