use uefi::{
    Result,
    runtime::{Daylight, Time, TimeCapabilities, TimeError, get_time_and_caps},
};

#[derive(Clone, Copy, Debug)]
pub struct KernelTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanosecond: u32,
    pub daylight: Daylight,
    pub time_zone: Option<i16>,
}

pub extern "win64" fn get_uefi_time() -> Result<KernelTime> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let (time, _caps) = get_time_and_caps()?;
        Ok(KernelTime {
            year: time.year(),
            month: time.month(),
            day: time.day(),
            hour: time.hour(),
            minute: time.minute(),
            second: time.second(),
            nanosecond: time.nanosecond(),
            daylight: time.daylight(),
            time_zone: time.time_zone(),
        })
    })
}

// pub fn s_time(time: &Time) -> Result<()> {
//     return unsafe {set_time(time)};
// }

pub type GetTimeFn = extern "win64" fn() -> Result<KernelTime>;
// pub type SetTimeFn = fn() -> Result<()>;
