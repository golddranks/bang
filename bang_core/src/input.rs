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
    fn relax(&mut self) {
        match self {
            KeyState::Pressed => *self = KeyState::Down,
            KeyState::Down => *self = KeyState::Down,
            KeyState::Released => *self = KeyState::Up,
            KeyState::Up => *self = KeyState::Up,
            KeyState::Tap => *self = KeyState::Up,
        }
    }

    pub fn down(&self) -> bool {
        matches!(self, KeyState::Down | KeyState::Pressed | KeyState::Tap)
    }

    pub fn up(&self) -> bool {
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
    pub fn relax(&mut self) {
        self.space.relax();
        self.up.relax();
        self.down.relax();
        self.left.relax();
        self.right.relax();
    }

    pub const fn new() -> Self {
        InputState {
            space: KeyState::Up,
            up: KeyState::Up,
            down: KeyState::Up,
            left: KeyState::Up,
            right: KeyState::Up,
        }
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
}

#[derive(Debug)]
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
            126 => Key::Up,
            125 => Key::Down,
            123 => Key::Left,
            124 => Key::Right,
            _ => Key::Other,
        }
    }
}
