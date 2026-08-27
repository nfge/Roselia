use core::{ops::Add, sync::atomic::AtomicU64};

use alloc::{fmt::format, format};
use spin::mutex::Mutex;

use crate::{
    logger::loglevel::LogLevel,
    ramfs::{is_valid, write_file},
    timer::sleep,
};
pub mod loglevel;

pub struct Logger {
    offset: u64,
}

pub static LOGGER: Mutex<Logger> = Mutex::new(Logger::new());

impl Logger {
    pub const fn new() -> Self {
        Self { offset: 0 }
    }
    pub fn write_into_log(&mut self, line: &str) {
        let data: &[u8] = line.as_bytes();
        let _ = is_valid("/kernel/log").unwrap();
        let _ = write_file("/kernel/log", self.offset as usize, data).unwrap();
        self.offset += data.len() as u64;
    }
    pub fn write_with_loglevel(&mut self,args: core::fmt::Arguments<'_>, level: LogLevel) -> core::fmt::Result {
        use core::fmt::Write;
        write!(self, "[ {} ] ", level)?;
        self.write_fmt(args)
    }
}

impl core::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_into_log(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let logger = &crate::logger::LOGGER;
        let _ = write!(logger.lock(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let logger = &crate::logger::LOGGER;
        let _ = logger.lock().write_with_loglevel(format_args!($($arg)*), crate::logger::loglevel::LogLevel::Info);
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        let logger = &crate::logger::LOGGER;
        let _ = logger.lock().write_with_loglevel(format_args!($($arg)*), crate::logger::loglevel::LogLevel::Warning);
    }};
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {{
        let logger = &crate::logger::LOGGER;
        let _ = logger.lock().write_with_loglevel(format_args!($($arg)*), crate::logger::loglevel::LogLevel::Error);
    }};
}

#[macro_export]
macro_rules! log_fail {
    ($($arg:tt)*) => {{
        let logger = &crate::logger::LOGGER;
        let _ = logger.lock().write_with_loglevel(format_args!($($arg)*), crate::logger::loglevel::LogLevel::Fail);
    }};
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        if cfg!(debug_assertions) {
            let logger = &crate::logger::LOGGER;
            let _ = logger.lock().write_with_loglevel(format_args!($($arg)*), crate::logger::loglevel::LogLevel::Debug);
        }
    }};
}
