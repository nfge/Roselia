use uefi::{Result, runtime::{Time, TimeCapabilities, get_time_and_caps}};

// pub struct STime {
//     pub get_time: GetTimeFn,
// }

pub fn get_uefi_time() -> Result<(Time,TimeCapabilities)> {
    return get_time_and_caps();
}

// pub fn s_time(time: &Time) -> Result<()> {
//     return unsafe {set_time(time)};
// }

pub type GetTimeFn = fn() -> Result<(Time,TimeCapabilities)>;
// pub type SetTimeFn = fn() -> Result<()>;