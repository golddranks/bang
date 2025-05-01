use bang_core::{alloc::Alloc, draw::DrawFrame, game::GameState, input::InputState};

pub fn test_frame_logic_panicking<'f>(
    _: &mut Alloc<'f>,
    _: &InputState,
    _: &mut GameState,
) -> DrawFrame<'f> {
    panic!("Oh no!")
}

#[cfg(feature = "export")]
bang_core::export_frame_logic!(test_frame_logic_panicking);
