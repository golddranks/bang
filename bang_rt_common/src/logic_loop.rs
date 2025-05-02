use std::ops::Not;

use bang_core::{Config, game::GameState};

use crate::{
    alloc::AllocManager, draw::DrawSender, end::Ender, input::InputConsumer, load::FrameLogic,
    timer::Timer,
};

pub fn run<'l>(
    frame_logic: impl FrameLogic<'l>,
    mut input_consumer: InputConsumer,
    mut sender: DrawSender,
    mut alloc_manager: AllocManager,
    ender: &Ender,
    config: &Config,
) {
    let mut timer = Timer::new(config.logic_fps);
    let mut game_state = GameState::new();
    while ender.should_end().not() {
        let next_deadline = timer.wait_until_next();
        let keys = input_consumer.get_gathered(next_deadline);
        let alloc = alloc_manager.get_frame_alloc();
        let draw_frame = frame_logic.do_frame(alloc, keys, &mut game_state);
        let draw_frame = alloc.frame_val(draw_frame);
        sender.send(draw_frame);
        game_state.end_frame();
    }
    // To ensure that notify_end gets called in case of should_end being set "silently" by a signal handler
    ender.soft_quit();
    alloc_manager.wait_until_cleanup();
}
