
pub const SLP_EN: u16 = 1 << 13;
pub const SLP_TYP_SHIFT: u16 = 10;

pub fn find_sleep_type(name: &[u8; 4], aml: &[u8]) -> Option<(u8, u8)> {
    

    let pos = aml.windows(4).position(|w| w == name)?;

    if pos == 0 || aml[pos - 1] != 0x08 {
        return None;
    }

    let mut i = pos + 4;
    if *aml.get(i)? != 0x12 {
        return None;
    }
    i += 1;

    let lead = *aml.get(i)?;
    let extra = (lead >> 6) as usize;
    i += 1 + extra;
    i += 1;

    let read_elem = |i: &mut usize| -> Option<u8> {
        let v = match *aml.get(*i)? {
            0x0A => {
                *i += 1;
                *aml.get(*i)?
            }
            v => v,
        };
        *i += 1;
        Some(v)
    };

    let slp_typa = read_elem(&mut i)?;
    let slp_typb = read_elem(&mut i)?;
    Some((slp_typa, slp_typb))
}
