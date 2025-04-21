pub mod keys;
pub mod num;

pub use keys::KeysState;

use num::F;

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
pub enum Cmd {
    DrawSQuads {
        texture: TextureID,
        pos: &'static [Pos],
    },
}

#[derive(Debug)]
#[repr(C)]
pub struct DrawFrame {
    cmds: &'static [Cmd],
}

impl DrawFrame {
    pub fn debug_dummies(_dummy: &[(f32, f32)]) -> Self {
        DrawFrame {
            cmds: vec![Cmd::DrawSQuads {
                texture: TextureID(0),
                pos: &[], // TODO
            }]
            .leak(),
        }
    }
}

pub type FrameLogicExternFn = extern "Rust" fn(&KeysState, &mut GameState) -> DrawFrame;
pub type FrameLogicFn = fn(&KeysState, &mut GameState) -> DrawFrame;

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
        pub extern "Rust" fn frame_logic_no_mangle(
            input: &KeysState,
            game_state: &mut GameState,
        ) -> DrawFrame {
            let implementation: bang_core::FrameLogicFn = $impl;
            implementation(input, game_state)
        }
    };
}
