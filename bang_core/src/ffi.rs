use std::ffi::CStr;

use arena::Id;

use crate::{Config, alloc::Mem, draw::DrawFrame, input::InputState};

pub type FnUpdateRaw = for<'f> fn(
    alloc: &mut Mem<'f>,
    input: &InputState,
    rt: &mut RtCtx,
    erased_state: *mut Erased,
) -> DrawFrame<'f>;

pub type FnInitRaw = for<'f> fn(mem: &mut Mem<'f>, rt: &mut RtCtx) -> LogicInitReturn;

pub struct Erased;

#[derive(Clone, Copy, Debug)]
pub struct SendableErasedPtr(pub *mut Erased);

unsafe impl Send for SendableErasedPtr {}

impl SendableErasedPtr {
    pub fn wrap<T>(mut ptr: Box<T>) -> Self {
        SendableErasedPtr(&raw mut *ptr as *mut Erased)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct LogicInitReturn {
    pub logic_state: *mut Erased,
    pub config: Config,
}

pub struct Tex;

#[derive(Debug)]
#[repr(C)]
pub struct RtCtx {
    pub frame: u64,
    pub rt_kind: RtKind,
    pub load_textures_ptr: for<'f> fn(&mut Self, &[&str], &mut Mem<'f>) -> &'f [Id<Tex>],
    pub rt_state: SendableErasedPtr,
}

impl RtCtx {
    pub fn load_textures<'f>(&mut self, tex: &[&str], mem: &mut Mem<'f>) -> &'f [Id<Tex>] {
        (self.load_textures_ptr)(self, tex, mem)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub enum RtKind {
    Test,
    MacOS,
    TUI,
}

impl RtCtx {
    pub fn end_frame(&mut self) {
        self.frame += 1;
    }
}

pub trait Logic: Send + Sized {
    type S;

    fn new() -> Self {
        unimplemented!()
    }

    #[allow(unused)]
    fn init(&self, mem: &mut Mem<'_>, ctx: &mut RtCtx) -> (Self::S, Config) {
        unimplemented!()
    }

    fn init_raw(&self, mem: &mut Mem<'_>, ctx: &mut RtCtx) -> LogicInitReturn {
        let (state, config) = Self::init(&Self::new(), mem, ctx);
        LogicInitReturn {
            logic_state: &raw mut *Box::new(state) as *mut Erased,
            config,
        }
    }

    #[allow(unused)]
    fn update<'f>(
        &self,
        mem: &mut Mem<'f>,
        input: &InputState,
        ctx: &mut RtCtx,
        state: &mut Self::S,
    ) -> DrawFrame<'f> {
        unimplemented!()
    }

    fn update_raw<'f>(
        &self,
        mem: &mut Mem<'f>,
        input: &InputState,
        ctx: &mut RtCtx,
        erased_state: *mut Erased,
    ) -> DrawFrame<'f> {
        let state = unsafe { &mut *(erased_state as *mut Self::S) };
        Self::update(self, mem, input, ctx, state)
    }
}

#[macro_export]
macro_rules! export_logic {
    ($impl:ident) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn logic_update<'f>(
            alloc: &mut $crate::alloc::Mem<'f>,
            input: &$crate::input::InputState,
            rt: &mut $crate::ffi::RtCtx,
            state: *mut $crate::ffi::Erased,
        ) -> $crate::draw::DrawFrame<'f> {
            let slf = <$impl as $crate::ffi::Logic>::new();
            <$impl as $crate::ffi::Logic>::update_raw(&slf, alloc, input, rt, state)
        }

        #[unsafe(no_mangle)]
        pub extern "Rust" fn logic_init<'f>(
            mem: &mut Mem<'f>,
            rt: &mut $crate::ffi::RtCtx,
        ) -> $crate::ffi::LogicInitReturn {
            let slf = <$impl as $crate::ffi::Logic>::new();
            <$impl as $crate::ffi::Logic>::init_raw(&slf, mem, rt)
        }
    };
}

pub const LOGIC_INIT_SYM: &CStr = c"logic_init";
pub const LOGIC_UPDATE_SYM: &CStr = c"logic_update";
