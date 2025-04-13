use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self},
};

mod draw;
mod error;
mod objc;
mod timer;
mod win;

use timer::Timer;

fn logic_loop(end: &AtomicBool) {
    let mut timer = Timer::new(120);
    while end.load(Ordering::Acquire).not() {
        //dbg!(timer.fps());
        timer.wait_until_next();
    }
}

fn main() {
    let end = AtomicBool::new(false);
    thread::scope(|s| {
        s.spawn(|| logic_loop(&end));
        win::init(&end); // Runs in main thread because of AppKit limitations
    });

    println!("Bye!");
}
