use bang_core::{Alloc, DrawFrame, GameState, InputState, export_frame_logic};

fn frame_logic<'frame>(
    alloc: &mut Alloc<'frame>,
    _input: &InputState,
    _game_state: &mut GameState,
) -> DrawFrame<'frame> {
    dbg!("frame_logic");
    DrawFrame::debug_dummies(alloc, &[(-100.0, -100.0), (100.0, 100.0)])
}

export_frame_logic!(frame_logic);
