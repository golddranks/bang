use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self},
};

mod draw;
mod error;
mod keys;
mod objc;
mod timer;
mod win;

use keys::{KeyStateManager, KeysState};
use timer::Timer;

fn logic_loop(end: &AtomicBool) {
    let mut keys = Box::new(KeysState::new());
    let mut timer = Timer::new(120);
    while end.load(Ordering::Acquire).not() {
        timer.wait_until_next();
        keys = KeyStateManager::state_swap(keys);
        dbg!(&keys);
    }
}

fn main() {
    objc::init_objc();

    let end = AtomicBool::new(false);
    thread::scope(|s| {
        s.spawn(|| logic_loop(&end));
        win::init(&end); // Runs in main thread because of AppKit limitations
    });

    println!("Bye!");
}
