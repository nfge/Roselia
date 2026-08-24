use core::fmt::Display;


#[derive(Debug,Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fail
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Warning => write!(f, "Warning"),
            LogLevel::Error => write!(f, "Error"),
            LogLevel::Fail => write!(f, "FAIL"),
            LogLevel::Debug => write!(f, "Debug")
        }
    }
}