use std::{
    ffi::{CStr, c_char, c_int, c_void},
    ptr::NonNull,
};

use bang_core::{
    alloc::Mem,
    draw::DrawFrame,
    ffi::{
        Erased, FnInitRaw, FnUpdateRaw, LOGIC_INIT_SYM, LOGIC_UPDATE_SYM, Logic, LogicInitReturn,
        RtCtx,
    },
    input::InputState,
};

use crate::{die, error::OrDie};

unsafe extern "C" {
    safe fn dlopen(path: *const c_char, mode: c_int) -> Option<NonNull<c_void>>;
    safe fn dlsym(handle: NonNull<c_void>, symbol: *const c_char) -> Option<NonNull<c_void>>;
}

const RTLD_LAZY: c_int = 1;

pub fn dyn_load_logic(lib: &CStr) -> DynLoadedLogic {
    let lib_ptr = dlopen(lib.as_ptr(), RTLD_LAZY).or_(die!("Failed to load library"));
    let update_ptr = dlsym(lib_ptr, LOGIC_UPDATE_SYM.as_ptr())
        .or_(die!("Failed to find symbol: {:?}", LOGIC_UPDATE_SYM));
    let init_ptr = dlsym(lib_ptr, LOGIC_INIT_SYM.as_ptr())
        .or_(die!("Failed to find symbol: {:?}", LOGIC_INIT_SYM));
    let update = unsafe { std::mem::transmute::<NonNull<c_void>, FnUpdateRaw>(update_ptr) };
    let init = unsafe { std::mem::transmute::<NonNull<c_void>, FnInitRaw>(init_ptr) };
    DynLoadedLogic {
        update_raw_ptr: update,
        init_raw_ptr: init,
    }
}

pub struct DynLoadedLogic {
    update_raw_ptr: FnUpdateRaw,
    init_raw_ptr: FnInitRaw,
}

impl Logic for DynLoadedLogic {
    type S = Erased;

    fn init_raw(&self, mem: &mut Mem<'_>, rt: &mut RtCtx) -> LogicInitReturn {
        (self.init_raw_ptr)(mem, rt)
    }

    fn update_raw<'f>(
        &self,
        alloc: &mut Mem<'f>,
        input: &InputState,
        rt: &mut RtCtx,
        erased_state: *mut Erased,
    ) -> DrawFrame<'f> {
        (self.update_raw_ptr)(alloc, input, rt, erased_state)
    }
}

#[cfg(test)]
pub mod tests {
    use std::ptr::null_mut;

    use super::*;
    use arena::Arena;
    use bang_core::ffi::{RtKind, SendableErasedPtr};
    use test_normal_dylib::TestLogic;

    #[test]
    fn test_inline() {
        let mut arenac = Arena::default();
        let mut alloc = Mem::new(arenac.fresh_arena(1));
        let input_state = InputState::default();
        let mut ctx = RtCtx {
            frame: 0,
            rt_kind: RtKind::Test,
            load_textures_ptr: crate::runtime::tests::load_textures,
            rt_state: SendableErasedPtr(null_mut()),
        };
        let mut state = Erased;
        TestLogic.update_raw(&mut alloc, &input_state, &mut ctx, &raw mut state);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_load() {
        let mut arenac = Arena::default();
        let dyn_logic = dyn_load_logic(c"../target/tests/libtest_normal_dylib.dylib");
        let mut alloc = Mem::new(arenac.fresh_arena(1));
        let input_state = InputState::default();
        let mut ctx = RtCtx {
            frame: 0,
            rt_kind: RtKind::Test,
            load_textures_ptr: crate::runtime::tests::load_textures,
            rt_state: SendableErasedPtr(null_mut()),
        };
        let mut state = Erased;
        dyn_logic.update_raw(&mut alloc, &input_state, &mut ctx, &raw mut state);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "Failed to load library")]
    fn test_lib_not_found() {
        dyn_load_logic(c"nonexisting.dylib");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    #[should_panic(expected = "Failed to find symbol")]
    fn test_lib_missing_symbol() {
        dyn_load_logic(c"../target/tests/libtest_symbol_missing_dylib.dylib");
    }
}
