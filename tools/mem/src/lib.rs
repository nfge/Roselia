#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(ptr1: *const u8, ptr2: *const u8, count: usize) -> i32 {
    let s1 = core::slice::from_raw_parts(ptr1, count);
    let s2 = core::slice::from_raw_parts(ptr2, count);

    for i in 0..count {
        if s1[i] != s2[i] {
            return s1[i] as i32 - s2[i] as i32;
        }
    }
    0
}

pub fn strncmp(str1: &str, str2: &str, num: usize) -> i32 {
    let mut s1 = str1.chars();
    let mut s2 = str2.chars();
    for _ in 0..num {
        match (s1.next(), s2.next()) {
            (Some(a), Some(b)) => {
                if a != b {
                    return a as i32 - b as i32;
                }
            }
            (Some(a), None) => return a as i32,
            (None, Some(b)) => return -(b as i32),
            (None, None) => return 0,
        }
    }
    0
}
