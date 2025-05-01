use std::{sync::Mutex, time::Instant};

use bang_core::input::{InputState, Key, KeyState};

#[derive(Debug)]
pub struct InputConsumer<'l> {
    shared: &'l SharedInputState,
    consuming: Box<InputState>,
}

pub fn make_input_tools(shared: &mut SharedInputState) -> (InputGatherer, InputConsumer) {
    let shared = &*shared; // Take as unique, but make shared to prevent other references
    (
        InputGatherer { shared },
        InputConsumer {
            shared,
            consuming: Box::new(InputState::new()),
        },
    )
}

impl<'l> InputConsumer<'l> {
    pub fn get_gathered(&mut self, next_deadline: Instant) -> &InputState {
        {
            let state = &mut **self.shared.gather.lock().expect("UNREACHABLE");
            *self.consuming = state.current.clone();
            state.current.relax_and_merge(&mut state.next);
            state.deadline = next_deadline;
        }
        self.consuming.as_ref()
    }
}

#[derive(Debug)]
pub struct SharedInputState {
    gather: Mutex<Box<InnerSharedInputState>>,
}

impl Default for SharedInputState {
    fn default() -> Self {
        Self {
            gather: Mutex::new(Box::new(InnerSharedInputState {
                current: InputState::new(),
                next: InputState::new(),
                deadline: Instant::now(),
            })),
        }
    }
}

#[derive(Debug)]
struct InnerSharedInputState {
    current: InputState,
    next: InputState,
    deadline: Instant,
}

#[derive(Debug)]
pub struct InputGatherer<'l> {
    shared: &'l SharedInputState,
}

impl<'l> InputGatherer<'l> {
    pub fn update(&mut self, key: Key, state: KeyState, timestamp: Instant) {
        let shared_state = &mut **self.shared.gather.lock().expect("UNREACHABLE");

        if timestamp > shared_state.deadline {
            shared_state.next.update(key, state);
        } else {
            shared_state.current.update(key, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_input() {
        let mut shared = SharedInputState::default(); // Deadline is set
        let (mut gatherer, mut consumer) = make_input_tools(&mut shared);

        gatherer.update(Key::Left, KeyState::Pressed, Instant::now());
        gatherer.update(Key::Right, KeyState::Released, Instant::now());

        // The deadline in the past, so the updates are not sent

        // Deadline is reset
        let gathered = consumer.get_gathered(Instant::now() + Duration::from_secs(1));
        gatherer.update(Key::Space, KeyState::Pressed, Instant::now());

        assert_eq!(gathered.left, KeyState::Up);
        assert_eq!(gathered.right, KeyState::Up);

        let gathered = consumer.get_gathered(Instant::now());

        assert_eq!(gathered.left, KeyState::Pressed);
        assert_eq!(gathered.right, KeyState::Released);
        assert_eq!(gathered.space, KeyState::Pressed);

        let gathered = consumer.get_gathered(Instant::now());

        assert_eq!(gathered.left, KeyState::Down); // KeyState relaxing happens
        assert_eq!(gathered.right, KeyState::Up);
        assert_eq!(gathered.space, KeyState::Down);
    }
}
