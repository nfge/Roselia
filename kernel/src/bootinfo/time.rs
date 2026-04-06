use uefi::{Result, runtime::{Time,TimeCapabilities}};


pub type GetTimeFn = fn() -> Result<(Time,TimeCapabilities)>;