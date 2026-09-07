use core::fmt::Arguments;

use crate::{log_debug, log_err, log_fail, log_info, log_warn, logger::loglevel::LogLevel};

pub extern "Rust" fn log(args: Arguments, level: LogLevel) {
    match level {
        LogLevel::Info => log_info!("{}", args),
        LogLevel::Warning => log_warn!("{}", args),
        LogLevel::Error => log_err!("{}", args),
        LogLevel::Fail => log_fail!("{}", args),
        LogLevel::Debug => log_debug!("{}", args),
    }
}