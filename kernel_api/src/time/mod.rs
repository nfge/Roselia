use uefi::runtime::Daylight;

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