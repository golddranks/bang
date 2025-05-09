use crate::{alloc::Alloc, draw::DrawFrame, game::GameState, input::InputState};

pub type FrameLogicExternFn =
    for<'f> extern "Rust" fn(&mut Alloc<'f>, &InputState, &mut GameState) -> DrawFrame<'f>;

pub type FrameLogicFn = for<'f> fn(&mut Alloc<'f>, &InputState, &mut GameState) -> DrawFrame<'f>;

#[macro_export]
macro_rules! frame_logic_sym_name {
    () => {
        "frame_logic_000"
    };
}

#[macro_export]
macro_rules! config_sym_name {
    () => {
        "config_000"
    };
}

#[macro_export]
macro_rules! export_frame_logic {
    ($impl:ident) => {
        #[unsafe(export_name = $crate::frame_logic_sym_name!())]
        pub extern "Rust" fn frame_logic_no_mangle<'f>(
            alloc: &mut Alloc<'f>,
            input: &InputState,
            game_state: &mut GameState,
        ) -> DrawFrame<'f> {
            let implementation: $crate::ffi::FrameLogicFn = $impl;
            implementation(alloc, input, game_state)
        }
    };
}

#[macro_export]
macro_rules! export_config {
    ($config:expr) => {
        #[unsafe(export_name = $crate::config_sym_name!())]
        static __CONFIG: Config = $config;
    };
}
