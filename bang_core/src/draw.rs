use std::slice;

use crate::alloc::Alloc;

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

impl Cmd<'_> {
    pub fn as_bytes(pos: &[ScreenPos]) -> &[u8] {
        unsafe { slice::from_raw_parts(pos.as_ptr() as *const u8, size_of_val(pos)) }
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
        draw::{Cmd, DrawFrame},
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
        assert_eq!(Cmd::as_bytes(&[ScreenPos { x: 1.0, y: 2.0 }]), xy);
    }
}
