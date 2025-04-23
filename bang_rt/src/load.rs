use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    ptr::NonNull,
};

use bang_core::{
    alloc::Alloc, draw::DrawFrame, ffi::FrameLogicExternFn, frame_logic_sym_name, game::GameState,
    input::InputState,
};

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
    let sym_name = CString::new(frame_logic_sym_name!()).expect("UNREACHABLE");
    let Some(frame_logic_ptr) = dlsym(lib_ptr, sym_name.as_ptr()) else {
        panic!("Failed to find symbol: {:?}", sym_name);
    };
    unsafe { std::mem::transmute::<NonNull<c_void>, FrameLogicExternFn>(frame_logic_ptr) }
}

trait FrameLogic<'f> {
    fn call(
        self,
        alloc: &mut Alloc<'f>,
        input: &InputState,
        game_state: &mut GameState,
    ) -> DrawFrame<'f>;
}

impl<'f> FrameLogic<'f> for FrameLogicExternFn<'f> {
    fn call(
        self,
        alloc: &mut Alloc<'f>,
        input: &InputState,
        game_state: &mut GameState,
    ) -> DrawFrame<'f> {
        self(alloc, input, game_state)
    }
}
