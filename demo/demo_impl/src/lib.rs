use bang_core::{
    Config,
    alloc::Mem,
    draw::{Cmd, DrawFrame, ScreenPos, TextureID},
    export_config, export_frame_logic,
    game::GameState,
    input::InputState,
};

pub fn frame_logic<'f>(
    mem: &mut Mem<'f>,
    input: &InputState,
    game_state: &mut GameState,
) -> DrawFrame<'f> {
    let fr = game_state.frame as f32 / 10.0;
    if input.space.down() {
        DrawFrame::debug_dummies(&[(1.0, fr), (-50.0, -50.0), (50.0, 50.0)], mem)
    } else {
        let pos_bubu = ScreenPos::slice(&[(0.0, 0.0)], mem);
        let pos_toge = ScreenPos::slice(
            &[(100.0 - fr * 2.0, -80.0), (-130.0, 40.0), (140.0, 0.0)],
            mem,
        );
        let pos_lima = ScreenPos::slice(&[(-100.0, 30.0)], mem);
        let bubu = Cmd::draw_s_quads(TextureID(1), pos_bubu);
        let toge = Cmd::draw_s_quads(TextureID(2), pos_toge);
        let lima = Cmd::draw_s_quads(TextureID(3), pos_lima);
        let cmds = mem.slice(&[bubu, toge, lima]);
        DrawFrame::with_cmds(cmds, mem.alloc_seq)
    }
}

pub const CONFIG: Config = Config {
    name: "Demo",
    resolution: (320, 200),
    logic_fps: 60,
    scale: 6,
};

export_frame_logic!(frame_logic);
export_config!(CONFIG);
