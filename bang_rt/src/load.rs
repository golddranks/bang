use std::ffi::{CStr, c_char, c_int, c_void};

use bang_core::FrameLogicExternFn;

unsafe extern "C" {
    safe fn dlopen(path: *const c_char, mode: c_int) -> *mut c_void;
    safe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

const RTLD_LAZY: c_int = 1;

pub fn get_frame_logic(libname: &str) -> FrameLogicExternFn {
    let libname = format!("lib{}.dylib\0", libname);
    let libname = CStr::from_bytes_with_nul(libname.as_bytes()).expect("UNREACHABLE");
    let lib_ptr = dlopen(libname.as_ptr(), RTLD_LAZY);
    if lib_ptr.is_null() {
        panic!("Failed to load library");
    }
    let frame_logic_ptr = dlsym(lib_ptr, c"frame_logic".as_ptr());
    if frame_logic_ptr.is_null() {
        panic!("Failed to find frame_logic symbol");
    }
    unsafe { std::mem::transmute::<*mut c_void, FrameLogicExternFn>(frame_logic_ptr) }
}
