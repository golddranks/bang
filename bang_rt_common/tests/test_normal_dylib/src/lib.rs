use bang_core::{alloc::Alloc, draw::DrawFrame, game::GameState, input::InputState};

pub fn test_frame_logic_normal<'f>(
    alloc: &mut Alloc<'f>,
    _: &InputState,
    _: &mut GameState,
) -> DrawFrame<'f> {
    DrawFrame {
        alloc_seq: alloc.alloc_seq,
        cmds: &[],
    }
}

#[cfg(feature = "export")]
bang_core::export_frame_logic!(test_frame_logic_normal);
