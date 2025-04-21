use bang_core::{DrawFrame, GameState, InputState, export_frame_logic};

fn frame_logic(_input: &InputState, _game_state: &mut GameState) -> DrawFrame {
    // TODO
    dbg!("frame_logic");
    DrawFrame::debug_dummies(&[])
}

export_frame_logic!(frame_logic);
