pub mod alloc;
pub mod draw;
pub mod ffi;
pub mod game;
pub mod input;
pub mod num;

#[derive(Debug, Clone)]
pub struct Config {
    pub name: &'static str,
    pub resolution: (u32, u32),
    pub logic_fps: u64,
}
