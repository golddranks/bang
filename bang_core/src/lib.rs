pub mod keys;
pub mod num;

use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
};

pub use keys::InputState;

use num::{F, f_f32};

#[derive(Debug)]
#[repr(C)]
struct Vec2D {
    x: F,
    y: F,
}

#[derive(Debug)]
#[repr(C)]
pub struct Acc(Vec2D);

#[derive(Debug)]
#[repr(C)]
pub struct Vel(Vec2D);

#[derive(Debug)]
#[repr(C)]
pub struct Pos(Vec2D);

#[derive(Debug)]
#[repr(C)]
pub struct TextureID(u64);

#[derive(Debug)]
#[repr(C)]
pub enum Cmd<'frame> {
    DrawSQuads {
        texture: TextureID,
        pos: &'frame [Pos],
    },
}

#[derive(Debug)]
#[repr(C)]
pub struct DrawFrame<'frame> {
    cmds: &'frame [Cmd<'frame>],
}

impl<'frame> DrawFrame<'frame> {
    pub fn debug_dummies(alloc: &mut Alloc<'frame>, dummy: &[(f32, f32)]) -> Self {
        let mut pos_buf = alloc.frame_buf(dummy.len());
        for &(x, y) in dummy {
            pos_buf.push(Pos(Vec2D {
                x: f_f32(x),
                y: f_f32(y),
            }));
        }
        let pos = pos_buf.to_slice();
        let mut cmd_buf = alloc.frame_buf(1);
        cmd_buf.push(Cmd::DrawSQuads {
            texture: TextureID(0),
            pos,
        });
        let cmds = cmd_buf.to_slice();
        DrawFrame { cmds }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct AllocManager {
    pool: Vec<Alloc<'static>>,
}

impl AllocManager {
    pub fn new() -> Self {
        AllocManager { pool: Vec::new() }
    }

    pub fn frame_alloc<'frame>(&mut self) -> Alloc<'frame> {
        if let Some(mut alloc) = self.pool.pop() {
            alloc.reset();
            alloc
        } else {
            Alloc::new()
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Alloc<'frame> {
    buf: Vec<u8>,
    _marker: PhantomData<&'frame ()>,
}

pub struct Buf<'a, T> {
    init: usize,
    buf: &'a mut [MaybeUninit<T>],
}

impl<'a, T> Buf<'a, T> {
    fn push(&mut self, value: T) {
        if self.init < self.buf.len() {
            self.buf[self.init].write(value);
            self.init += 1;
        }
    }

    fn to_slice(&self) -> &'a [T] {
        let slice = slice_from_raw_parts(self.buf.as_ptr() as *const T, self.init);
        unsafe { &*slice }
    }
}

impl<'frame> Alloc<'frame> {
    pub fn frame_buf<T>(&mut self, len: usize) -> Buf<'frame, T> {
        self.buf.reserve(len * size_of::<T>()); // FIXME: This is going to invalidate earlier buffers
        let byte_buf = self.buf.spare_capacity_mut();
        let buf_ptr = slice_from_raw_parts_mut(byte_buf.as_mut_ptr() as *mut MaybeUninit<T>, len);
        let typed_buf = unsafe { &mut *buf_ptr };
        Buf {
            init: 0,
            buf: typed_buf,
        }
    }

    pub fn new() -> Self {
        Alloc {
            buf: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn reset(&mut self) {
        self.buf.clear();
    }
}

pub type FrameLogicExternFn<'frame> =
    extern "Rust" fn(&mut Alloc<'frame>, &InputState, &mut GameState) -> DrawFrame<'frame>;
pub type FrameLogicFn<'frame> =
    fn(&mut Alloc<'frame>, &InputState, &mut GameState) -> DrawFrame<'frame>;

#[derive(Debug, Default)]
#[repr(C)]
pub struct GameState {}

impl GameState {
    pub fn new() -> Self {
        GameState {}
    }
}

#[macro_export]
macro_rules! export_frame_logic {
    ($impl:ident) => {
        #[unsafe(export_name = "frame_logic")]
        pub extern "Rust" fn frame_logic_no_mangle<'frame>(
            alloc: &mut Alloc<'frame>,
            input: &InputState,
            game_state: &mut GameState,
        ) -> DrawFrame<'frame> {
            let implementation: bang_core::FrameLogicFn = $impl;
            implementation(alloc, input, game_state)
        }
    };
}
