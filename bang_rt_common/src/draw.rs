use std::{
    ops::Not,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
    u16,
};

use bang_core::draw::{AsBytes, DRAW_FRAME_DUMMY, DrawFrame};

use crate::alloc::AllocRetirer;

#[derive(Debug)]
pub struct SharedDrawState<'l> {
    fresh: AtomicPtr<DrawFrame<'l>>,
}

impl<'l> Default for SharedDrawState<'l> {
    fn default() -> Self {
        Self {
            fresh: AtomicPtr::new(null_mut()),
        }
    }
}

#[derive(Debug)]
pub struct DrawSender<'l> {
    shared: &'l SharedDrawState<'l>,
    retirer: &'l AllocRetirer<'l>,
}

#[derive(Debug)]
pub struct DrawReceiver<'l> {
    shared: &'l SharedDrawState<'l>,
    retirer: &'l AllocRetirer<'l>,
    fresh: &'l DrawFrame<'l>,
}

pub fn make_draw_tools<'l>(
    shared: &'l mut SharedDrawState<'l>,
    retirer: &'l mut AllocRetirer<'l>,
) -> (DrawSender<'l>, DrawReceiver<'l>) {
    let shared = &*shared;
    let retirer = &*retirer;
    let sender = DrawSender { shared, retirer };
    let receiver = DrawReceiver {
        shared,
        retirer,
        fresh: &DRAW_FRAME_DUMMY,
    };
    (sender, receiver)
}

impl<'l> DrawSender<'l> {
    pub fn send<'f>(&mut self, frame: &'f mut DrawFrame<'f>) {
        #[allow(clippy::unnecessary_cast)]
        let frame = &raw mut *frame as *mut DrawFrame<'l>;
        let missed_frame = self.shared.fresh.swap(frame, Ordering::Release);
        if missed_frame.is_null().not() {
            let missed_frame = unsafe { &*missed_frame };
            self.retirer.retire_early(missed_frame.alloc_seq);
        }
    }
}

impl<'l> DrawReceiver<'l> {
    pub fn get_fresh<'s>(&'s mut self) -> &'s DrawFrame<'s> {
        let freshest = self.shared.fresh.swap(null_mut(), Ordering::Acquire);
        if freshest.is_null().not() {
            let retired_seq = self.fresh.alloc_seq;
            self.fresh = unsafe { &mut *freshest };
            self.retirer.retire_up_to(retired_seq);
        }
        self.fresh
    }

    pub fn has_fresh(&self) -> bool {
        self.shared.fresh.load(Ordering::Acquire).is_null().not()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct Color {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub a: u16,
}

unsafe impl AsBytes for Color {}

impl Color {
    pub const fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        let max = u16::MAX as f32;
        assert!(r >= 0.0 && r <= 1.0);
        assert!(g >= 0.0 && g <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);
        assert!(a >= 0.0 && a <= 1.0);
        Self {
            r: (r * max) as u16,
            g: (g * max) as u16,
            b: (b * max) as u16,
            a: (a * max) as u16,
        }
    }
}

pub struct PalTex {
    width: u32,
    height: u32,
    palette: Vec<Color>,
    data: Vec<u8>,
}

impl PalTex {
    pub fn from_bytes<const N: usize, const M: usize>(
        palette: &[(u8, Color)],
        bytes: &[[u8; N]; M],
    ) -> Self {
        let mut data = bytes.as_flattened().to_owned();
        let mut pal_idx_lookup = [0_u8; 256];
        let mut colors = Vec::with_capacity(palette.len());

        // Prepare the palette and the lookup table
        for (new_idx, (old_idx, color)) in palette.iter().enumerate() {
            colors.push(*color);
            pal_idx_lookup[*old_idx as usize] = new_idx as u8;
        }

        // Re-assign the palette indices to the data
        for i in 0..data.len() {
            data[i] = pal_idx_lookup[data[i] as usize];
        }

        Self {
            height: bytes.len() as u32,
            width: bytes[0].len() as u32,
            palette: colors,
            data,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn palette(&self) -> &[Color] {
        &self.palette
    }

    pub fn width(&self) -> usize {
        self.width as usize
    }

    pub fn height(&self) -> usize {
        self.height as usize
    }
}

#[cfg(test)]
mod tests {
    use bang_core::draw::Cmd;

    use super::*;
    use crate::alloc::{SharedAllocState, make_alloc_tools};

    #[test]
    fn test_draw() {
        let mut shared_draw = SharedDrawState::default();
        let mut shared_alloc = SharedAllocState::default();
        let (mut manager, mut retirer, cleanup) = make_alloc_tools(&mut shared_alloc);
        let (mut sender, mut receiver) = make_draw_tools(&mut shared_draw, &mut retirer);

        let mut alloc = manager.get_frame_alloc(); // Frame 1
        let mut frame = DrawFrame::debug_dummies(&mut alloc, &[(1.0, 2.0), (3.0, 4.0)]);
        sender.send(&mut frame);

        assert!(receiver.has_fresh()); // Actually fresh
        let fresh = receiver.get_fresh();
        assert_eq!(fresh.alloc_seq, 1);
        assert_eq!(fresh.cmds.len(), 1);
        assert!(matches!(fresh.cmds[0], Cmd::DrawSQuads { .. }));

        let fresh = receiver.get_fresh(); // The same as last time
        assert_eq!(fresh.alloc_seq, 1);

        let mut alloc = manager.get_frame_alloc(); // Frame 2
        let mut frame = DrawFrame::debug_dummies(&mut alloc, &[(1.0, 2.0), (3.0, 4.0)]);
        sender.send(&mut frame);

        let mut alloc = manager.get_frame_alloc(); // Frame 3
        let mut frame = DrawFrame::debug_dummies(&mut alloc, &[(1.0, 2.0), (3.0, 4.0)]);
        sender.send(&mut frame); // Retire early frame 2

        let fresh = receiver.get_fresh(); // Get frame 3
        assert_eq!(fresh.alloc_seq, 3);

        cleanup.cleanup();
        manager.wait_until_cleanup();
    }
}
