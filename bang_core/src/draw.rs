use crate::alloc::Alloc;

#[derive(Debug)]
#[repr(C)]
pub struct ScreenPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
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
    pub fn alloc_seq(&self) -> usize {
        self.alloc_seq
    }

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
            alloc_seq: alloc.get_alloc_seq(),
            cmds,
        }
    }
}
