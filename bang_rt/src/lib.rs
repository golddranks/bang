use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use bang_core::{FrameLogicFn, GameState};

mod draw;
mod error;
mod keys;
mod load;
mod objc;
mod timer;
mod win;

pub use keys::KeysState;

use keys::KeyStateManager;
use timer::Timer;
use win::Window;

fn logic_loop(end: &AtomicBool, frame_logic: FrameLogicFn) {
    let mut keys = Box::new(KeysState::new());
    let mut timer = Timer::new(120);
    let mut game_state = GameState::new();
    while end.load(Ordering::Acquire).not() {
        timer.wait_until_next();
        keys = KeyStateManager::state_swap(keys);
        let _frame = frame_logic(&keys, &mut game_state); // TODO
    }
}

static END: AtomicBool = AtomicBool::new(false);

pub fn start_runtime(libname: &str) {
    objc::init_objc();

    let frame_logic = load::get_frame_logic(libname);
    let window = Window::init(&END);

    thread::scope(|s| {
        s.spawn(|| logic_loop(&END, frame_logic));
        window.run(); // Runs in main thread because of AppKit limitations
    });

    println!("Bye!");
}
