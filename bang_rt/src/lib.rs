use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use bang_core::{alloc::AllocManager, game::GameState};

mod draw;
mod error;
mod input;
mod load;
mod objc;
mod timer;
mod win;

use draw::{DrawSender, SharedDrawState, new_draw_pair};
use input::{InputConsumer, SharedInputState, new_input_pair};
use load::FrameLogic;
use timer::Timer;
use win::Window;

pub use load::as_frame_logic;

fn logic_loop<'l>(
    frame_logic: impl FrameLogic<'l>,
    mut consumer: InputConsumer,
    mut sender: DrawSender,
) {
    let mut timer = Timer::new(120);
    let mut game_state = GameState::new();
    let mut alloc_manager = AllocManager::new();
    while should_end().not() {
        let next_deadline = timer.wait_until_next();
        let keys = consumer.consume_gathered(next_deadline);
        let mut alloc = alloc_manager.frame_alloc();
        let draw_frame = frame_logic.call(&mut alloc, &keys, &mut game_state);
        let draw_frame = alloc.frame(draw_frame);
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
    let (gatherer, consumer) = new_input_pair(&mut shared_input_state);
    let mut shared_draw_state = SharedDrawState::new();
    let (sender, receiver) = new_draw_pair(&mut shared_draw_state);

    let window = Window::init(gatherer, receiver);

    thread::scope(|s| {
        s.spawn(|| logic_loop(frame_logic, consumer, sender));
        window.run(); // Runs in main thread because of AppKit limitations
    });

    println!("Bye!");
}
