use bang_core::{
    Config,
    alloc::{Id, Mem},
    draw::{Cmd, DrawFrame, ScreenPos},
    export_logic,
    ffi::{Logic, RtCtx, RtKind, Tex},
    input::InputState,
};

pub struct DemoLogic;

impl Logic for DemoLogic {
    type S = State;

    fn new() -> Self {
        Self
    }

    fn init(&self, mem: &mut Mem, ctx: &mut RtCtx) -> (State, Config) {
        let tex_ids = ctx.load_textures(
            &[
                "assets/paltex/bubu.paltex",
                "assets/paltex/toge.paltex",
                "assets/paltex/lima.paltex",
            ],
            mem,
        );
        (
            State {
                bubu_tex: tex_ids[0],
                toge_tex: tex_ids[1],
                lima_tex: tex_ids[2],
            },
            Config {
                name: "Demo",
                resolution: (320, 200),
                logic_fps: if ctx.rt_kind == RtKind::TUI { 10 } else { 60 },
                scale: 6,
            },
        )
    }

    fn update<'f>(
        &self,
        mem: &mut Mem<'f>,
        input: &InputState,
        ctx: &mut RtCtx,
        state: &mut State,
    ) -> DrawFrame<'f> {
        let fr = ctx.frame as f32 / 10.0;
        if input.space.down() {
            DrawFrame::debug_dummies(&[(1.0, fr), (-50.0, -50.0), (50.0, 50.0)], mem)
        } else {
            let pos_bubu = ScreenPos::slice(&[(0.0, 0.0)], mem);
            let pos_toge = ScreenPos::slice(
                &[(100.0 - fr * 2.0, -80.0), (-130.0, 40.0), (140.0, 0.0)],
                mem,
            );
            let pos_lima = ScreenPos::slice(&[(-100.0, 30.0)], mem);
            let bubu = Cmd::draw_s_quads(state.bubu_tex, pos_bubu);
            let toge = Cmd::draw_s_quads(state.toge_tex, pos_toge);
            let lima = Cmd::draw_s_quads(state.lima_tex, pos_lima);
            let cmds = mem.slice(&[bubu, toge, lima]);
            DrawFrame::with_cmds(cmds, mem.alloc_seq)
        }
    }
}

#[derive(Debug)]
pub struct State {
    bubu_tex: Id<Tex>,
    toge_tex: Id<Tex>,
    lima_tex: Id<Tex>,
}

export_logic!(DemoLogic);
