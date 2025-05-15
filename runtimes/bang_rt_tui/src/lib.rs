mod draw;
mod input;
mod win;

use std::ptr::null_mut;

use bang_core::{
    Config,
    ffi::{RtCtx, RtKind, SendableErasedPtr},
};
use bang_rt_common::{draw::DrawReceiver, end::Ender, input::InputGatherer, runtime::Runtime};
use win::Window;

pub struct TuiRT;

const LOOP_MS: u64 = 33; // Input and output

impl Runtime for TuiRT {
    type Window<'a> = Window<'a>;

    fn init_rt(&self) {}

    fn init_win<'l>(
        &self,
        rt_ctx: &mut RtCtx,
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
        config: &'l Config,
    ) -> Self::Window<'l> {
        ender.install_global_signal_handler();
        Window::init(rt_ctx, input_gatherer, draw_receiver, ender, config)
    }

    fn run(win: &mut Self::Window<'_>) {
        win.run();
    }

    fn notify_end(_: &Ender) {}

    fn new_ctx(&self) -> RtCtx {
        RtCtx {
            frame: 0,
            rt_kind: RtKind::TUI,
            load_textures_ptr: draw::load_textures,
            rt_state: SendableErasedPtr(null_mut()),
        }
    }
}
