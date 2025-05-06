use std::slice::{from_raw_parts, from_raw_parts_mut};

use crate::alloc::Alloc;

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
    pub fn draw_squads(texture: TextureID, pos: &[ScreenPos], alloc: &mut Alloc<'f>) -> Self {
        let mut pos_vec = alloc.frame_vec();
        for p in pos {
            pos_vec.push(*p);
        }
        let pos = pos_vec.into_slice();
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
    pub fn with_cmds(cmds: &'f [Cmd], alloc: &mut Alloc<'f>) -> Self {
        DrawFrame {
            alloc_seq: alloc.alloc_seq,
            cmds,
        }
    }

    pub fn debug_dummies(dummies: &[(f32, f32)], alloc: &mut Alloc<'f>) -> Self {
        let mut pos_vec = Vec::new();
        for &(x, y) in dummies {
            pos_vec.push(ScreenPos { x, y });
        }
        let cmd = Cmd::draw_squads(TextureID(0), &pos_vec, alloc);
        let mut cmd_vec = alloc.frame_vec();
        cmd_vec.push(cmd);
        Self::with_cmds(cmd_vec.into_slice(), alloc)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        alloc::Alloc,
        draw::{AsBytes, Cmd, DrawFrame},
    };

    use super::ScreenPos;

    #[test]
    fn test_debug_dummies() {
        let mut alloc = Alloc::default();
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
