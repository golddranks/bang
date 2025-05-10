use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    ptr::NonNull,
};

use bang_core::{
    Config, alloc::Alloc, config_sym_name, draw::DrawFrame, ffi::FrameLogicExternFn,
    frame_logic_sym_name, game::GameState, input::InputState,
};

use crate::{die, error::OrDie};

unsafe extern "C" {
    safe fn dlopen(path: *const c_char, mode: c_int) -> Option<NonNull<c_void>>;
    safe fn dlsym(handle: NonNull<c_void>, symbol: *const c_char) -> Option<NonNull<c_void>>;
}

const RTLD_LAZY: c_int = 1;

pub fn get_symbols(lib: &CStr) -> (FrameLogicExternFn, Config) {
    let lib_ptr = dlopen(lib.as_ptr(), RTLD_LAZY).or_(die!("Failed to load library"));
    let frame_logic_sym_name = CString::new(frame_logic_sym_name!()).expect("UNREACHABLE");
    let frame_logic_ptr = dlsym(lib_ptr, frame_logic_sym_name.as_ptr())
        .or_(die!("Failed to find symbol: {frame_logic_sym_name:?}"));
    let config_sym_name = CString::new(config_sym_name!()).expect("UNREACHABLE");
    let config_ptr = dlsym(lib_ptr, config_sym_name.as_ptr())
        .or_(die!("Failed to find symbol: {config_sym_name:?}"));
    let frame_logic =
        unsafe { std::mem::transmute::<NonNull<c_void>, FrameLogicExternFn>(frame_logic_ptr) };
    let config = unsafe { std::mem::transmute::<NonNull<c_void>, &Config>(config_ptr) };
    (frame_logic, config.clone())
}

pub trait FrameLogic: Send {
    fn do_frame<'f>(
        &self,
        alloc: &mut Alloc<'f>,
        input: &InputState,
        game_state: &mut GameState,
    ) -> DrawFrame<'f>;
}

impl FrameLogic for FrameLogicExternFn {
    fn do_frame<'f>(
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

impl<F> FrameLogic for InlinedFrameLogic<F>
where
    F: Send + for<'f> Fn(&mut Alloc<'f>, &InputState, &mut GameState) -> DrawFrame<'f>,
{
    fn do_frame<'f>(
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
    use arena::ArenaContainer;
    use test_normal_dylib::test_frame_logic_normal;

    #[test]
    fn test_inline() {
        let mut arenac = ArenaContainer::default();
        let mut alloc = Alloc::new(arenac.new_arena(1));
        let input_state = InputState::default();
        let mut game_state = GameState::default();
        let frame_logic = InlinedFrameLogic::new(test_frame_logic_normal);
        frame_logic.do_frame(&mut alloc, &input_state, &mut game_state);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_load() {
        let mut arenac = ArenaContainer::default();
        let (frame_logic, _config) = get_symbols(c"../target/tests/libtest_normal_dylib.dylib");
        let mut alloc = Alloc::new(arenac.new_arena(1));
        let input_state = InputState::default();
        let mut game_state = GameState::default();
        frame_logic.do_frame(&mut alloc, &input_state, &mut game_state);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "Failed to load library")]
    fn test_lib_not_found() {
        get_symbols(c"nonexisting.dylib");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "Failed to find symbol")]
    fn test_lib_missing_symbol() {
        get_symbols(c"../target/tests/libtest_symbol_missing_dylib.dylib");
    }
}
