use core::{ops::Add, sync::atomic::AtomicU64};

use alloc::{fmt::format, format};

use crate::{logger::loglevel::LogLevel, ramfs::{is_valid, write_file}, timer::sleep};
pub mod loglevel;

pub struct Logger ;

pub static LOGGER_OFFSET: AtomicU64 = AtomicU64::new(0);

impl Logger {
    pub fn write_into_log(line: &str) {
        let data: &[u8] = line.as_bytes(); 
        let _ = is_valid("/kernel/log").unwrap();
        let _ = write_file("/kernel/log", LOGGER_OFFSET.load(core::sync::atomic::Ordering::Relaxed) as usize, data).unwrap();
        LOGGER_OFFSET.fetch_add(data.len() as u64, core::sync::atomic::Ordering::Relaxed);
    }
    pub fn write_with_loglevel(line: &str, level: LogLevel) {
        let rdata = format!("[ {level} ] {}", line);
        let data = rdata.as_bytes();
        let _ = is_valid("/kernel/log").unwrap();
        let _ = write_file("/kernel/log", LOGGER_OFFSET.load(core::sync::atomic::Ordering::Relaxed) as usize, data).unwrap();
        LOGGER_OFFSET.fetch_add(data.len() as u64, core::sync::atomic::Ordering::Relaxed);
    }
}

impl core::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Self::write_into_log(s);
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

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        let _ = crate::logger::Logger::write_with_loglevel($($arg)*, crate::logger::loglevel::LogLevel::Info);
    };
}
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        let _ = crate::logger::Logger::write_with_loglevel($($arg)*, crate::logger::loglevel::LogLevel::Warning);
    };
}
#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        let _ = crate::logger::Logger::write_with_loglevel($($arg)*, crate::logger::loglevel::LogLevel::Error);
    };
}
#[macro_export]
macro_rules! log_fail {
    ($($arg:tt)*) => {
        let _ = crate::logger::Logger::write_with_loglevel($($arg)*, crate::logger::loglevel::LogLevel::Fail);
    };
}
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            let _ = crate::logger::Logger::write_with_loglevel($($arg)*, crate::logger::loglevel::LogLevel::Debug);
        }
    };
}