use std::{
    collections::VecDeque,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

pub use bang_core::keys::{InputState, Key, KeyState};

use crate::{objc::wrappers::NSTimeInterval, timer::Timer};

static KEY_STATE_GATHER: AtomicPtr<InputState> = AtomicPtr::new(null_mut());

#[derive(Debug)]
pub struct KeyStateManager {
    event_queue: VecDeque<(Key, KeyState)>,
}

impl KeyStateManager {
    pub fn new() -> Self {
        KEY_STATE_GATHER.store(
            Box::into_raw(Box::new(InputState::new())),
            Ordering::Release,
        );
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

    pub fn state_swap(mut old_state: Box<InputState>) -> Box<InputState> {
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
