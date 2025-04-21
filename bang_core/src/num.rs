// Fixed decimal standard width: 24 bit signed 2's complement integer + 8 bit fractional part
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct F(i32);
// Fixed decimal double width: 48 bit signed 2's complement integer + 16 bit fractional part
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FD(i64);

const F_MAX: F = F(i32::MAX >> 8);
const F_MIN: F = F(i32::MIN >> 8);

pub const fn f_i32(num: i32) -> F {
    debug_assert!(num <= F_MAX.0);
    debug_assert!(num >= F_MIN.0);
    F(num << 8)
}

pub const fn f_f32(num: f32) -> F {
    debug_assert!(num <= F_MAX.0 as f32);
    debug_assert!(num >= F_MIN.0 as f32);
    F((num * 256.0) as i32)
}

#[test]
fn test_f_i32() {
    assert_eq!(f_i32(123), F(123 << 8));
    assert_eq!(f_i32(8388607), F(8388607 << 8));
    assert_eq!(f_i32(-8388608), F(-8388608 << 8));
}

#[test]
fn test_f_f32() {
    assert_eq!(f_f32(123.0), F(123 << 8));
    assert_eq!(f_f32(123.5), F(247 << 7));
    assert_eq!(f_f32(8388607.0), F(8388607 << 8));
    assert_eq!(f_f32(-8388608.0), F(-8388608 << 8));
}

#[test]
#[should_panic]
fn test_f_i32_overflow() {
    f_i32(8388608);
}

#[test]
#[should_panic]
fn test_f_i32_underflow() {
    f_i32(-8388609);
}

#[test]
#[should_panic]
fn test_f_f32_overflow() {
    f_f32(8388608.0);
}

#[test]
#[should_panic]
fn test_f_f32_underflow() {
    f_f32(-8388609.0);
}

/* TODO
pub const fn f_str(str: &str) -> F {
    todo!()
}

macro_rules! n {
    ($lit:literal) => {
        $($tt)*
    };
}
 */
