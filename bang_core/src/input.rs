#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyState {
    #[default]
    Up,
    Down,
    Pressed,
    Released,
    Tap,
}

impl KeyState {
    const fn relax(&mut self) {
        *self = match self {
            KeyState::Pressed => KeyState::Down,
            KeyState::Released => KeyState::Up,
            KeyState::Tap => KeyState::Up,
            _ => *self,
        };
    }

    const fn merge(&mut self, next: &mut KeyState) {
        self.relax();
        if next.edge() {
            *self = *next;
            *next = KeyState::Up;
        }
    }

    pub const fn edge(&self) -> bool {
        matches!(self, KeyState::Pressed | KeyState::Released | KeyState::Tap)
    }

    pub const fn down(&self) -> bool {
        matches!(self, KeyState::Down | KeyState::Pressed | KeyState::Tap)
    }

    pub const fn up(&self) -> bool {
        matches!(self, KeyState::Up | KeyState::Released | KeyState::Tap)
    }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct InputState {
    pub space: KeyState,
    pub up: KeyState,
    pub down: KeyState,
    pub left: KeyState,
    pub right: KeyState,
}

impl InputState {
    pub const fn new() -> Self {
        InputState {
            space: KeyState::Up,
            up: KeyState::Up,
            down: KeyState::Up,
            left: KeyState::Up,
            right: KeyState::Up,
        }
    }

    pub const fn relax_and_merge(&mut self, next: &mut InputState) {
        self.space.merge(&mut next.space);
        self.up.merge(&mut next.up);
        self.down.merge(&mut next.down);
        self.left.merge(&mut next.left);
        self.right.merge(&mut next.right);
    }

    pub fn update(&mut self, key: Key, state: KeyState) {
        let old_state = match key {
            Key::Space => &mut self.space,
            Key::Up => &mut self.up,
            Key::Down => &mut self.down,
            Key::Left => &mut self.left,
            Key::Right => &mut self.right,
            Key::Other => return,
        };
        if *old_state == KeyState::Pressed && state == KeyState::Released {
            *old_state = KeyState::Tap;
        } else {
            *old_state = state;
        }
    }

    pub fn get(&self, key: Key) -> KeyState {
        match key {
            Key::Space => self.space,
            Key::Up => self.up,
            Key::Down => self.down,
            Key::Left => self.left,
            Key::Right => self.right,
            Key::Other => KeyState::Up,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
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

    pub fn from_ascii(ascii: u8) -> Self {
        match ascii {
            b' ' => Key::Space,
            _ => Key::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InputState, Key, KeyState};

    #[test]
    fn test_relax_and_merge() {
        let mut state = InputState::new();
        let mut neutral = InputState::new();

        let all_keys = &[Key::Down, Key::Left, Key::Right, Key::Up, Key::Space];

        for key in all_keys {
            let state = state.get(*key);
            assert!(state.up());
        }

        for key in all_keys {
            state.update(*key, KeyState::Pressed);
        }

        for key in all_keys {
            let state = state.get(*key);
            assert!(state.down());
            assert_eq!(state, KeyState::Pressed);
        }

        state.relax_and_merge(&mut neutral);

        for key in all_keys {
            let state = state.get(*key);
            assert!(state.down());
            assert_eq!(state, KeyState::Down);
        }

        state.relax_and_merge(&mut neutral);

        for key in all_keys {
            let state = state.get(*key);
            assert!(state.down());
            assert_eq!(state, KeyState::Down);
        }

        for key in all_keys {
            state.update(*key, KeyState::Released);
        }

        for key in all_keys {
            let state = state.get(*key);
            assert!(state.up());
            assert_eq!(state, KeyState::Released);
        }

        state.relax_and_merge(&mut neutral);

        for key in all_keys {
            let state = state.get(*key);
            assert_eq!(state, KeyState::Up);
            assert!(state.up());
        }

        for key in all_keys {
            state.update(*key, KeyState::Tap);
        }

        for key in all_keys {
            let state = state.get(*key);
            assert_eq!(state, KeyState::Tap);
            assert!(state.up());
            assert!(state.down());
        }

        state.relax_and_merge(&mut neutral);

        for key in all_keys {
            let state = state.get(*key);
            assert_eq!(state, KeyState::Up);
            assert!(state.up());
        }

        for key in all_keys {
            state.update(*key, KeyState::Up);
        }

        for key in all_keys {
            let state = state.get(*key);
            assert_eq!(state, KeyState::Up);
            assert!(state.up());
        }

        state.relax_and_merge(&mut neutral);

        for key in all_keys {
            let state = state.get(*key);
            assert_eq!(state, KeyState::Up);
            assert!(state.up());
        }

        let mut next = InputState::new();

        for key in all_keys {
            next.update(*key, KeyState::Pressed);
        }

        for key in all_keys {
            let state = next.get(*key);
            assert_eq!(state, KeyState::Pressed);
        }

        state.relax_and_merge(&mut next);

        for key in all_keys {
            let state = state.get(*key);
            assert_eq!(state, KeyState::Pressed);
            let state = next.get(*key);
            assert_eq!(state, KeyState::Up);
        }
    }

    #[test]
    fn test_tap() {
        let mut state = InputState::new();

        let all_keys = &[
            Key::Down,
            Key::Left,
            Key::Right,
            Key::Up,
            Key::Space,
            Key::Other,
        ];

        for key in all_keys {
            state.update(*key, KeyState::Pressed);
        }

        for key in all_keys {
            state.update(*key, KeyState::Released);
        }

        for key in all_keys {
            let state = state.get(*key);
            if *key == Key::Other {
                assert_eq!(state, KeyState::Up);
            } else {
                assert_eq!(state, KeyState::Tap);
            }
        }
    }

    #[test]
    fn test_convert() {
        for i in 0..256 {
            let _ = Key::from_code(i);
        }

        assert_eq!(Key::from_ascii(b' '), Key::Space);
        assert_eq!(Key::from_ascii(b'a'), Key::Other); // TODO
    }
}
