use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use bang_core::{
    alloc::AllocManager,
    ffi::{FrameLogicExternFn, FrameLogicFn},
    game::GameState,
};

mod draw;
mod error;
mod input;
mod load;
mod objc;
mod timer;
mod win;

use draw::{DrawSender, SharedDrawState, new_draw_pair};
use input::{InputConsumer, SharedInputState, new_input_pair};
use timer::Timer;
use win::Window;

fn logic_loop(frame_logic: FrameLogicFn, mut consumer: InputConsumer, sender: DrawSender) {
    let mut timer = Timer::new(120);
    let mut game_state = GameState::new();
    let mut alloc_manager = AllocManager::new();
    while should_end().not() {
        let next_deadline = timer.wait_until_next();
        let keys = consumer.consume_gathered(next_deadline);
        let mut alloc = alloc_manager.frame_alloc();
        let frame = frame_logic(&mut alloc, &keys, &mut game_state);
        sender.send(frame);
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

pub fn start_runtime_with_static(frame_logic: FrameLogicExternFn) {
    main_loops(frame_logic);
}

fn main_loops(frame_logic: FrameLogicExternFn) {
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
