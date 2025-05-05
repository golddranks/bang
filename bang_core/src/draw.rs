use std::slice::from_raw_parts;

use crate::alloc::Alloc;

/// # Safety
/// This trait is safe to implement for types that don't have internal mutability (UnsafeCell)
/// within their memory layout, and don't contain any padding bytes. The types implementing this trait
/// should also preferably have a layout defined with repr(C) to ensure stability across rustc versions.
pub unsafe trait AsBytes {
    fn as_byte_ptr(&self) -> *const u8 {
        self as *const Self as *const u8
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts(self.as_byte_ptr(), size_of_val(self)) }
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
#[repr(C)]
pub struct TextureID(u64);

#[derive(Debug)]
#[repr(C)]
pub enum Cmd<'f> {
    DrawSQuads {
        texture: TextureID,
        pos: &'f [ScreenPos],
    },
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
    pub fn debug_dummies(alloc: &mut Alloc<'f>, dummies: &[(f32, f32)]) -> Self {
        let mut pos_vec = alloc.frame_vec();
        for &(x, y) in dummies {
            pos_vec.push(ScreenPos { x, y });
        }
        let pos = pos_vec.into_slice();
        let mut cmd_vec = alloc.frame_vec();
        cmd_vec.push(Cmd::DrawSQuads {
            texture: TextureID(0),
            pos,
        });
        let cmds = cmd_vec.into_slice();
        DrawFrame {
            alloc_seq: alloc.alloc_seq,
            cmds,
        }
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
        let frame = DrawFrame::debug_dummies(&mut alloc, &dummies);
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
