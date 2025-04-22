use bang_core::{
    alloc::Alloc, draw::DrawFrame, export_frame_logic, game::GameState, input::InputState,
};

fn frame_logic<'f>(
    alloc: &mut Alloc<'f>,
    input: &InputState,
    _game_state: &mut GameState,
) -> DrawFrame<'f> {
    if input.space.down() {
        DrawFrame::debug_dummies(alloc, &[(-100.0, -100.0), (100.0, 100.0)])
    } else {
        DrawFrame::debug_dummies(alloc, &[(-50.0, -50.0), (50.0, 50.0)])
    }
}

export_frame_logic!(frame_logic);
