use std::{
    ops::Not,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self},
};

mod draw;
mod error;
mod keys;
mod num;
mod objc;
mod timer;
mod win;

use keys::{KeyStateManager, KeysState};
use num::{F, f_i32};
use timer::Timer;

struct Vec2D {
    x: F,
    y: F,
}

struct Acc(Vec2D);

struct Vel(Vec2D);

struct Pos(Vec2D);

struct TextureID(u64);

struct DrawSprite {
    pos: Pos,
    texture: TextureID,
}

struct Frame {
    sprites: Vec<DrawSprite>,
}

fn logic(input: &KeysState, game_state: &GameState) -> Frame {
    Frame {
        sprites: vec![DrawSprite {
            pos: Pos(Vec2D {
                x: f_i32(1),
                y: f_i32(2),
            }),
            texture: TextureID(0),
        }],
    }
}

struct GameState {}

impl GameState {
    fn new() -> Self {
        GameState {}
    }
}

fn logic_loop(end: &AtomicBool) {
    let mut keys = Box::new(KeysState::new());
    let mut timer = Timer::new(120);
    let mut game_state = GameState::new();
    while end.load(Ordering::Acquire).not() {
        timer.wait_until_next();
        keys = KeyStateManager::state_swap(keys);
        let frame = logic(&keys, &game_state);
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
