use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use bang_core::game::GameState;

use alloc::{AllocManager, SharedAllocState, new_alloc_pair};
use draw::{DrawReceiver, DrawSender, SharedDrawState, new_draw_pair};
use input::{InputConsumer, InputGatherer, SharedInputState, new_input_pair};
use load::FrameLogic;
use timer::Timer;

pub mod alloc;
pub mod draw;
pub mod error;
pub mod input;
pub mod load;
mod timer;

static END: AtomicBool = AtomicBool::new(false);

pub fn should_end() -> bool {
    END.load(Ordering::Acquire)
}

pub fn soft_quit() {
    END.store(true, Ordering::Release);
}

fn logic_loop<'l>(
    frame_logic: impl FrameLogic<'l>,
    mut input_consumer: InputConsumer,
    mut sender: DrawSender,
    mut alloc_manager: AllocManager,
) {
    let mut timer = Timer::new(10); // TODO: make configurable
    let mut game_state = GameState::new();
    while should_end().not() {
        let next_deadline = timer.wait_until_next();
        let keys = input_consumer.get_gathered(next_deadline);
        let alloc = alloc_manager.get_frame_alloc();
        let draw_frame = frame_logic.call(alloc, keys, &mut game_state);
        let draw_frame = alloc.frame_val(draw_frame);
        sender.send(draw_frame);
    }
}

pub fn start_rt_dynamic<'l, 's, RT: Runtime>(libname: &'s str) {
    let frame_logic = load::get_frame_logic(libname);
    main_loops::<RT>(frame_logic);
}

pub fn start_rt_static<'l, RT: Runtime>(frame_logic: impl FrameLogic<'l>) {
    main_loops::<RT>(frame_logic);
}

pub fn main_loops<'l, RT: Runtime>(frame_logic: impl FrameLogic<'l>) {
    RT::init_rt();

    let mut shared_input_state = SharedInputState::default();
    let (input_gatherer, input_consumer) = new_input_pair(&mut shared_input_state);
    let mut shared_alloc_state = SharedAllocState::default();
    let (alloc_manager, alloc_retirer) = new_alloc_pair(&mut shared_alloc_state);
    let mut shared_draw_state = SharedDrawState::default();
    let (draw_sender, draw_receiver) = new_draw_pair(&mut shared_draw_state, alloc_retirer);

    let mut window = RT::init_win(input_gatherer, draw_receiver);

    thread::scope(|s| {
        s.spawn(|| logic_loop(frame_logic, input_consumer, draw_sender, alloc_manager));
        RT::run(&mut window); // Runs in main thread because of possible platform thread limitations
    });

    println!("Bye!");
}

pub trait Runtime {
    type Window<'a>;
    fn init_rt();
    fn init_win<'l>(
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
    ) -> Self::Window<'l>;
    fn run(win: &mut Self::Window<'_>);
}
