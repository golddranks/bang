#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum KeyState {
    Pressed,
    Down,
    Released,
    #[default]
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

    pub fn down(&self) -> bool {
        matches!(self, KeyState::Down | KeyState::Pressed)
    }

    pub fn up(&self) -> bool {
        matches!(self, KeyState::Up | KeyState::Released)
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
}
