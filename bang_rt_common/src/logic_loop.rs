use std::ops::Not;

use bang_core::{Config, alloc::Alloc, game::GameState, input::InputState};

use crate::{
    alloc::AllocManager, draw::DrawSender, end::Ender, input::InputConsumer, load::FrameLogic,
    timer::Timer,
};

fn with_frame_lifetime<'f>(
    frame_logic: &impl FrameLogic,
    input: &InputState,
    game_state: &mut GameState,
    sender: &mut DrawSender,
    alloc: &mut Alloc<'f>,
) {
    let draw_frame = frame_logic.do_frame(alloc, input, game_state);
    let draw_frame = alloc.val(draw_frame);
    sender.send_to_renderer(draw_frame);
    game_state.end_frame();
}

pub fn run(
    frame_logic: impl FrameLogic,
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
        let input = input_consumer.get_gathered(next_deadline);
        let mut alloc = alloc_manager.get_alloc();
        with_frame_lifetime(
            &frame_logic,
            input,
            &mut game_state,
            &mut sender,
            &mut alloc,
        );
    }
    // To ensure that notify_end gets called in case of should_end being set "silently" by a signal handler
    ender.soft_quit();
    alloc_manager.wait_until_cleanup();
}
