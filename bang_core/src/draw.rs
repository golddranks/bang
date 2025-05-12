use std::slice::{from_raw_parts, from_raw_parts_mut};

use crate::alloc::Mem;

/// # Safety
/// This trait is safe to implement for types that don't have
/// internal mutability (UnsafeCell) within their memory layout,
/// don't contain any padding bytes, and are valid for any bit patterns.
///
/// The types implementing this trait should also preferably have
/// a defined layout with repr(C) or repr(transparent) to ensure stability
/// across rustc versions and separately compiled dynamic libraries.
pub unsafe trait AsBytes {
    fn as_byte_ptr(&self) -> *const u8 {
        self as *const Self as *const u8
    }

    fn as_byte_ptr_mut(&mut self) -> *mut u8 {
        self as *mut Self as *mut u8
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts(self.as_byte_ptr(), size_of_val(self)) }
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { from_raw_parts_mut(self.as_byte_ptr_mut(), size_of_val(self)) }
    }
}

unsafe impl<T> AsBytes for [T] where T: AsBytes {}
unsafe impl<T, const N: usize> AsBytes for [T; N] where T: AsBytes {}

unsafe impl AsBytes for u8 {}
unsafe impl AsBytes for f32 {}
unsafe impl AsBytes for ScreenPos {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ScreenPos {
    pub x: f32,
    pub y: f32,
}

impl ScreenPos {
    pub fn slice<'f>(slice: &[(f32, f32)], mem: &mut Mem<'f>) -> &'f [ScreenPos] {
        mem.from_iter(slice.iter().map(|&(x, y)| ScreenPos { x, y }))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TextureID(pub u64);

#[derive(Debug)]
#[repr(C)]
pub enum Cmd<'f> {
    DrawSQuads {
        texture: TextureID,
        pos: &'f [ScreenPos],
    },
}

impl<'f> Cmd<'f> {
    pub fn draw_s_quads(texture: TextureID, pos: &'f [ScreenPos]) -> Self {
        Cmd::DrawSQuads { texture, pos }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct DrawFrame<'f> {
    pub alloc_seq: usize,
    pub cmds: &'f [Cmd<'f>],
}

pub static DRAW_FRAME_DUMMY: DrawFrame = DrawFrame {
    alloc_seq: 0,
    cmds: &[],
};

impl<'f> DrawFrame<'f> {
    pub fn with_cmds(cmds: &'f [Cmd], seq: usize) -> Self {
        DrawFrame {
            alloc_seq: seq,
            cmds,
        }
    }

    pub fn debug_dummies(dummies: &[(f32, f32)], mem: &mut Mem<'f>) -> Self {
        let pos = mem.from_iter(dummies.iter().map(|&(x, y)| ScreenPos { x, y }));
        let cmds = mem.slice(&[Cmd::draw_s_quads(TextureID(0), pos)]);
        Self::with_cmds(cmds, mem.alloc_seq)
    }
}

#[cfg(test)]
mod tests {
    use arena::Arena;

    use crate::{
        alloc::Mem,
        draw::{AsBytes, Cmd, DrawFrame},
    };

    use super::ScreenPos;

    #[test]
    fn test_debug_dummies() {
        let mut arena_container = Arena::default();
        let mut alloc = Mem::new(arena_container.fresh_arena(1));
        let dummies = [
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, -2.0),
            (-3.0, 3.0),
            (4.0, -4.0),
        ];
        let frame = DrawFrame::debug_dummies(&dummies, &mut alloc);
        assert_eq!(frame.alloc_seq, alloc.alloc_seq);
        assert_eq!(frame.cmds.len(), 1);
        match &frame.cmds[0] {
            &Cmd::DrawSQuads { texture, pos } => {
                assert_eq!(texture.0, 0);
                assert_eq!(pos.len(), 5);
                assert_eq!(pos[0].x, 0.0);
                assert_eq!(pos[0].y, 0.0);
                assert_eq!(pos[1].x, 1.0);
                assert_eq!(pos[1].y, 1.0);
            }
        }
    }

    #[test]
    fn test_as_bytes() {
        let x = 1.0_f32.to_le_bytes();
        let y = 2.0_f32.to_le_bytes();
        let xy = [x, y].concat();
        assert_eq!([ScreenPos { x: 1.0, y: 2.0 }].as_bytes(), xy);
    }
}
