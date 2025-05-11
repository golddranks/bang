use bang_core::{Config, alloc::Mem, draw::DrawFrame, game::GameState, input::InputState};

pub fn test_frame_logic_normal<'f>(
    alloc: &mut Mem<'f>,
    _: &InputState,
    _: &mut GameState,
) -> DrawFrame<'f> {
    DrawFrame {
        alloc_seq: alloc.alloc_seq,
        cmds: &[],
    }
}

pub const CONFIG: Config = Config {
    name: "Demo",
    resolution: (320, 200),
    logic_fps: 60,
    scale: 1,
};

#[cfg(feature = "export")]
bang_core::export_frame_logic!(test_frame_logic_normal);

#[cfg(feature = "export")]
bang_core::export_config!(CONFIG);
