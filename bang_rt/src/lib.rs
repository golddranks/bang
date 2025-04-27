use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use bang_core::game::GameState;

mod alloc;
mod draw;
mod error;
mod input;
mod load;
mod objc;
mod timer;
mod win;

use alloc::{AllocManager, SharedAllocState, new_alloc_pair};
use draw::{DrawSender, SharedDrawState, new_draw_pair};
use input::{InputConsumer, SharedInputState, new_input_pair};
use load::FrameLogic;
use timer::Timer;
use win::Window;

pub use load::as_frame_logic;

fn logic_loop<'l>(
    frame_logic: impl FrameLogic<'l>,
    mut input_consumer: InputConsumer,
    mut sender: DrawSender,
    mut alloc_manager: AllocManager,
) {
    let mut timer = Timer::new(1);
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

static END: AtomicBool = AtomicBool::new(false);

pub fn should_end() -> bool {
    END.load(Ordering::Acquire)
}

pub fn soft_quit() {
    END.store(true, Ordering::Release);
    Window::soft_quit();
}

pub fn start_runtime_with_dynamic(libname: &str) {
    let frame_logic = load::get_frame_logic(libname);
    main_loops(frame_logic);
}

pub fn start_runtime_with_static<'l>(frame_logic: impl FrameLogic<'l>) {
    main_loops(frame_logic);
}

fn main_loops<'l>(frame_logic: impl FrameLogic<'l>) {
    objc::init_objc();

    let mut shared_input_state = SharedInputState::new();
    let (input_gatherer, input_consumer) = new_input_pair(&mut shared_input_state);
    let mut shared_alloc_state = SharedAllocState::new();
    let (alloc_manager, alloc_retirer) = new_alloc_pair(&mut shared_alloc_state);
    let mut shared_draw_state = SharedDrawState::new();
    let (draw_sender, draw_receiver) = new_draw_pair(&mut shared_draw_state, alloc_retirer);

    let window = Window::init(input_gatherer, draw_receiver);

    thread::scope(|s| {
        s.spawn(|| logic_loop(frame_logic, input_consumer, draw_sender, alloc_manager));
        window.run(); // Runs in main thread because of AppKit limitations
    });

    println!("Bye!");
}
