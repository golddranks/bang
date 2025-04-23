use std::{collections::VecDeque, mem, sync::Mutex, time::Instant};

pub use bang_core::input::{InputState, Key, KeyState};

#[derive(Debug)]
pub struct SharedInputState {
    gather: Mutex<Box<(InputState, Instant)>>,
}

impl SharedInputState {
    pub fn new() -> Self {
        Self {
            gather: Mutex::new(Box::new((InputState::new(), Instant::now()))),
        }
    }
}

#[derive(Debug)]
pub struct InputConsumer<'l> {
    shared: &'l SharedInputState,
    consuming: Box<InputState>,
}

#[derive(Debug)]
pub struct InputGatherer<'l> {
    shared: &'l SharedInputState,
    pending: VecDeque<(Key, KeyState)>,
}

pub fn new_input_pair(shared: &mut SharedInputState) -> (InputGatherer, InputConsumer) {
    let shared = &*shared; // Take as unique, but make shared to prevent other references
    (
        InputGatherer {
            shared,
            pending: VecDeque::new(),
        },
        InputConsumer {
            shared,
            consuming: Box::new(InputState::new()),
        },
    )
}

impl<'l> InputGatherer<'l> {
    pub fn update(&mut self, key: Key, state: KeyState, timestamp: Instant) {
        let (input_state, deadline) = &mut **self.shared.gather.lock().expect("UNREACHABLE");

        if timestamp > *deadline {
            self.pending.push_back((key, state));
        } else {
            for (key, state) in self.pending.drain(..) {
                input_state.update(key, state);
            }
            input_state.update(key, state);
        }
    }
}

impl<'l> InputConsumer<'l> {
    pub fn consume_gathered(&mut self, next_deadline: Instant) -> &InputState {
        {
            let (gathered, deadline) = &mut **self.shared.gather.lock().expect("UNREACHABLE");
            *self.consuming = gathered.clone();
            self.consuming.relax();
            mem::swap(gathered, &mut self.consuming);
            *deadline = next_deadline;
        }
        self.consuming.as_ref()
    }
}
