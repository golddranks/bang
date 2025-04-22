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
mod keys;
mod load;
mod objc;
mod timer;
mod win;

pub use keys::InputState;

use keys::KeyStateManager;
use timer::Timer;
use win::Window;

fn logic_loop(end: &AtomicBool, frame_logic: FrameLogicFn) {
    let mut keys = Box::new(InputState::new());
    let mut timer = Timer::new(120);
    let mut game_state = GameState::new();
    let mut alloc_manager = AllocManager::new();
    while end.load(Ordering::Acquire).not() {
        timer.wait_until_next();
        keys = KeyStateManager::state_swap(keys);
        let mut alloc = alloc_manager.frame_alloc();
        let _frame = frame_logic(&mut alloc, &keys, &mut game_state); // TODO
    }
}

static END: AtomicBool = AtomicBool::new(false);

pub fn start_runtime_with_dynamic(libname: &str) {
    let frame_logic = load::get_frame_logic(libname);
    main_loops(frame_logic);
}

pub fn start_runtime_with_static(frame_logic: FrameLogicExternFn) {
    main_loops(frame_logic);
}

fn main_loops(frame_logic: FrameLogicExternFn) {
    objc::init_objc();

    let window = Window::init(&END);

    thread::scope(|s| {
        s.spawn(|| logic_loop(&END, frame_logic));
        window.run(); // Runs in main thread because of AppKit limitations
    });

    println!("Bye!");
}
