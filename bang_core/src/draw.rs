use crate::alloc::Alloc;

#[derive(Debug)]
#[repr(C)]
pub struct ScreenPos {
    x: f32,
    y: f32,
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
    cmds: &'f [Cmd<'f>],
}

impl<'f> DrawFrame<'f> {
    pub fn dummy() -> Self {
        Self { cmds: &[] }
    }

    pub fn debug_dummies(alloc: &mut Alloc<'f>, dummies: &[(f32, f32)]) -> Self {
        let mut pos_vec = alloc.frame_vec();
        for &(x, y) in dummies {
            pos_vec.as_vec().push(ScreenPos { x, y });
        }
        let pos = pos_vec.into_slice();
        let mut cmd_vec = alloc.frame_vec();
        cmd_vec.as_vec().push(Cmd::DrawSQuads {
            texture: TextureID(0),
            pos,
        });
        DrawFrame {
            cmds: cmd_vec.into_slice(),
        }
    }
}
