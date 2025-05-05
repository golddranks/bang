use bang_core::{
    Config, alloc::Alloc, draw::DrawFrame, export_config, export_frame_logic, game::GameState,
    input::InputState,
};

pub fn frame_logic<'f>(
    alloc: &mut Alloc<'f>,
    input: &InputState,
    game_state: &mut GameState,
) -> DrawFrame<'f> {
    let fr = game_state.frame as f32 / 10.0;
    if input.space.down() {
        DrawFrame::debug_dummies(alloc, &[(1.0, fr), (-50.0, -50.0), (50.0, 50.0)])
    } else {
        DrawFrame::debug_dummies(
            alloc,
            &[(-100.0 + fr, -80.0), (10.0 - fr * 3.0, 20.0), (49.0, 85.0)],
        )
    }
}

pub const CONFIG: Config = Config {
    name: "Demo",
    resolution: (320, 200),
    logic_fps: 60,
};

export_frame_logic!(frame_logic);
export_config!(CONFIG);
