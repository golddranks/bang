use crate::num::F;

#[derive(Debug)]
#[repr(C)]
struct Vec2D {
    x: F,
    y: F,
}

#[derive(Debug)]
#[repr(C)]
pub struct Acc(Vec2D);

#[derive(Debug)]
#[repr(C)]
pub struct Vel(Vec2D);

#[derive(Debug)]
#[repr(C)]
pub struct Pos(Vec2D);

#[derive(Debug, Default)]
#[repr(C)]
pub struct GameState {
    pub frame: u64,
}

impl GameState {
    pub fn new() -> Self {
        GameState { frame: 0 }
    }

    pub fn end_frame(&mut self) {
        self.frame += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::GameState;

    #[test]
    fn test_game_state() {
        let _ = GameState::new();
    }
}
