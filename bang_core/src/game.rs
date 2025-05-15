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

#[cfg(test)]
mod tests {}
