use std::ops::Not;

use bang_core::{
    Config,
    alloc::Mem,
    ffi::{Erased, Logic, RtCtx, SendableErasedPtr},
    input::InputState,
};

use crate::{
    alloc::AllocManager, draw::DrawSender, end::Ender, input::InputConsumer, timer::Timer,
};

pub struct RunArgs<'l, L> {
    pub logic: L,
    pub rt_ctx: &'l mut RtCtx,
    pub state: SendableErasedPtr,
    pub input_consumer: InputConsumer<'l>,
    pub sender: DrawSender<'l>,
    pub alloc_manager: AllocManager<'l>,
    pub ender: &'l Ender,
    pub config: &'l Config,
}

fn with_frame_lifetime<'f, L: Logic>(
    logic: &L,
    input: &InputState,
    rt_ctx: &mut RtCtx,
    state: *mut Erased,
    sender: &mut DrawSender,
    alloc: &mut Mem<'f>,
) {
    let draw_frame = logic.update_raw(alloc, input, rt_ctx, state);
    let draw_frame = alloc.val(draw_frame);
    sender.send_to_renderer(draw_frame);
    rt_ctx.end_frame();
}

pub fn run<'l>(mut args: RunArgs<'l, impl Logic>) {
    let mut timer = Timer::new(args.config.logic_fps);
    while args.ender.should_end().not() {
        let next_deadline = timer.wait_until_next();
        let input = args.input_consumer.get_gathered(next_deadline);
        let mut alloc = args.alloc_manager.get_alloc();
        with_frame_lifetime(
            &args.logic,
            input,
            args.rt_ctx,
            args.state.0,
            &mut args.sender,
            &mut alloc,
        );
    }
    // To ensure that notify_end gets called in case of should_end being set "silently" by a signal handler
    args.ender.soft_quit();
    args.alloc_manager.wait_until_cleanup();
}
