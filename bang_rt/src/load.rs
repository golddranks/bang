use std::{
    ffi::{CStr, c_char, c_int, c_void},
    ptr::NonNull,
};

use bang_core::FrameLogicExternFn;

use crate::error::OrDie;

unsafe extern "C" {
    safe fn dlopen(path: *const c_char, mode: c_int) -> Option<NonNull<c_void>>;
    safe fn dlsym(handle: NonNull<c_void>, symbol: *const c_char) -> Option<NonNull<c_void>>;
}

const RTLD_LAZY: c_int = 1;

pub fn get_frame_logic(libname: &str) -> FrameLogicExternFn {
    let libname = format!("lib{}.dylib\0", libname);
    let libname = CStr::from_bytes_with_nul(libname.as_bytes()).expect("UNREACHABLE");
    let lib_ptr = dlopen(libname.as_ptr(), RTLD_LAZY).or_die("Failed to load library");
    let frame_logic_ptr =
        dlsym(lib_ptr, c"frame_logic".as_ptr()).or_die("Failed to find frame_logic symbol");
    unsafe { std::mem::transmute::<NonNull<c_void>, FrameLogicExternFn>(frame_logic_ptr) }
}
