#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct F(i32); // Fixed decimal standard width: 1 bit sign, 23 bit integer + 8 bit fractional part
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FD(i64); // Fixed decimal double width: 1 bit sign, 47 bit integer + 16 bit fractional part

const F_MAX: F = F(i32::MAX >> 8);
const F_MIN: F = F(i32::MIN >> 8);

pub const fn f_i32(num: i32) -> F {
    debug_assert!(num <= F_MAX.0);
    debug_assert!(num >= F_MIN.0);
    F(num << 8)
}

#[test]
fn test_f_i32() {
    assert_eq!(f_i32(123), F(123 << 8));
    assert_eq!(f_i32(8388607), F(8388607 << 8));
    assert_eq!(f_i32(-8388608), F(-8388608 << 8));
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

pub const fn f_str(str: &str) -> F {
    todo!()
}

macro_rules! n {
    ($lit:literal) => {
        $($tt)*
    };
}
