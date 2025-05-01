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

pub fn get_frame_logic(lib: &CStr) -> FrameLogicExternFn {
    let lib_ptr = dlopen(lib.as_ptr(), RTLD_LAZY).or_die("Failed to load library");
    let sym_name = CString::new(frame_logic_sym_name!()).expect("UNREACHABLE");
    let Some(frame_logic_ptr) = dlsym(lib_ptr, sym_name.as_ptr()) else {
        panic!("Failed to find symbol: {sym_name:?}");
    };
    unsafe { std::mem::transmute::<NonNull<c_void>, FrameLogicExternFn>(frame_logic_ptr) }
}

pub trait FrameLogic<'f>: Send {
    fn do_frame(
        &self,
        alloc: &mut Alloc<'f>,
        input: &InputState,
        game_state: &mut GameState,
    ) -> DrawFrame<'f>;
}

impl<'f> FrameLogic<'f> for FrameLogicExternFn<'f> {
    fn do_frame(
        &self,
        alloc: &mut Alloc<'f>,
        input: &InputState,
        game_state: &mut GameState,
    ) -> DrawFrame<'f> {
        self(alloc, input, game_state)
    }
}

pub struct InlinedFrameLogic<F> {
    f: F,
}

impl<F> InlinedFrameLogic<F> {
    pub fn new(f: F) -> Self {
        InlinedFrameLogic { f }
    }
}

impl<'f, F> FrameLogic<'f> for InlinedFrameLogic<F>
where
    F: Send + Fn(&mut Alloc<'f>, &InputState, &mut GameState) -> DrawFrame<'f>,
{
    fn do_frame(
        &self,
        alloc: &mut Alloc<'f>,
        input: &InputState,
        game_state: &mut GameState,
    ) -> DrawFrame<'f> {
        (self.f)(alloc, input, game_state)
    }
}

#[cfg(test)]
pub mod tests {
    #![allow(unexpected_cfgs)]

    use super::*;
    use test_normal_dylib::test_frame_logic_normal;

    #[test]
    fn test_inline() {
        let mut alloc = Alloc::default();
        let input_state = InputState::default();
        let mut game_state = GameState::default();
        let frame_logic = InlinedFrameLogic::new(test_frame_logic_normal);
        frame_logic.do_frame(&mut alloc, &input_state, &mut game_state);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_load() {
        let frame_logic = get_frame_logic(c"../target/tests/libtest_normal_dylib.dylib");
        let mut alloc = Alloc::default();
        let input_state = InputState::default();
        let mut game_state = GameState::default();
        frame_logic.do_frame(&mut alloc, &input_state, &mut game_state);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "Failed to load library")]
    fn test_lib_not_found() {
        get_frame_logic(c"nonexisting.dylib");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "Failed to find symbol")]
    fn test_lib_missing_symbol() {
        get_frame_logic(c"../target/tests/libtest_symbol_missing_dylib.dylib");
    }
}
