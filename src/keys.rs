use std::{
    collections::VecDeque,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{objc::wrappers::NSTimeInterval, timer::Timer};

static KEY_STATE_GATHER: AtomicPtr<KeysState> = AtomicPtr::new(null_mut());

#[derive(Debug)]
pub struct KeyStateManager {
    event_queue: VecDeque<(Key, KeyState)>,
}

impl KeyStateManager {
    pub fn new() -> Self {
        KEY_STATE_GATHER.store(Box::into_raw(Box::new(KeysState::new())), Ordering::Release);
        KeyStateManager {
            event_queue: VecDeque::new(),
        }
    }

    pub fn update(&mut self, key: Key, state: KeyState, timestamp: NSTimeInterval) {
        if timestamp > Timer::deadline() {
            self.event_queue.push_back((key, state));
        } else {
            let state_ptr = KEY_STATE_GATHER.load(Ordering::Acquire);
            let keys_state = unsafe { &mut *state_ptr };
            for (key, state) in self.event_queue.drain(..) {
                keys_state.update(key, state);
            }
            keys_state.update(key, state);
        }
    }

    pub fn state_swap(mut old_state: Box<KeysState>) -> Box<KeysState> {
        while KEY_STATE_GATHER.load(Ordering::Acquire).is_null() {} // Wait for initialization
        let gathered =
            unsafe { Box::from_raw(KEY_STATE_GATHER.swap(null_mut(), Ordering::AcqRel)) };
        *old_state = *gathered.clone();
        old_state.relax();
        if KEY_STATE_GATHER
            .compare_exchange(
                null_mut(),
                Box::into_raw(old_state),
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_err()
        {
            unreachable!();
        }
        gathered
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeyState {
    Pressed,
    Down,
    Released,
    Up,
}

impl KeyState {
    fn relax(&mut self) {
        match self {
            KeyState::Pressed => *self = KeyState::Down,
            KeyState::Down => *self = KeyState::Down,
            KeyState::Released => *self = KeyState::Up,
            KeyState::Up => *self = KeyState::Up,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeysState {
    pub space: KeyState,
    pub up: KeyState,
    pub down: KeyState,
    pub left: KeyState,
    pub right: KeyState,
}

impl KeysState {
    fn relax(&mut self) {
        self.space.relax();
        self.up.relax();
        self.down.relax();
        self.left.relax();
        self.right.relax();
    }

    pub const fn new() -> Self {
        KeysState {
            space: KeyState::Up,
            up: KeyState::Up,
            down: KeyState::Up,
            left: KeyState::Up,
            right: KeyState::Up,
        }
    }

    pub fn update(&mut self, key: Key, state: KeyState) {
        match key {
            Key::Space => self.space = state,
            Key::Up => self.up = state,
            Key::Down => self.down = state,
            Key::Left => self.left = state,
            Key::Right => self.right = state,
            Key::Other => {}
        }
    }
}

#[derive(Debug)]
pub enum Key {
    Space,
    Up,
    Down,
    Left,
    Right,
    Other,
}

impl Key {
    pub fn from_code(code: u16) -> Self {
        match code {
            49 => Key::Space,
            126 => Key::Up,
            125 => Key::Down,
            123 => Key::Left,
            124 => Key::Right,
            _ => Key::Other,
        }
    }
}
