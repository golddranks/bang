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
    let _ = mem.slice(&[1, 2, 3, 4]);
    let fr = game_state.frame as f32 / 10.0;
    if input.space.down() {
        DrawFrame::debug_dummies(&[(1.0, fr), (-50.0, -50.0), (50.0, 50.0)], mem)
    } else {
        let pos_bubu = [ScreenPos { x: 0.0, y: 0.0 }];
        let pos_toge = [
            ScreenPos {
                x: 100.0 - fr * 2.0,
                y: -80.0,
            },
            ScreenPos {
                x: -130.0,
                y: 40.0 + fr,
            },
            ScreenPos { x: 140.0, y: 0.0 },
        ];
        let pos_lima = [ScreenPos { x: -100.0, y: 30.0 }];
        let bubu = Cmd::draw_squads(TextureID(1), pos_bubu.as_slice(), mem);
        let toge = Cmd::draw_squads(TextureID(2), pos_toge.as_slice(), mem);
        let lima = Cmd::draw_squads(TextureID(3), pos_lima.as_slice(), mem);
        let cmd_vec = mem.vec();
        cmd_vec.push(bubu);
        cmd_vec.push(toge);
        cmd_vec.push(lima);
        DrawFrame::with_cmds(cmd_vec.as_slice(), mem)
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
