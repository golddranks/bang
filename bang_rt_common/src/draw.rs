use std::{
    ops::Not,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use bang_core::draw::{DRAW_FRAME_DUMMY, DrawFrame};

pub use paltex::{Color, PalTex};

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
    sent_alloc_seq: usize,
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
    let sender = DrawSender {
        shared,
        retirer,
        sent_alloc_seq: 0,
    };
    let receiver = DrawReceiver {
        shared,
        retirer,
        fresh: &DRAW_FRAME_DUMMY,
    };
    (sender, receiver)
}

impl<'l> DrawSender<'l> {
    pub fn send<'f>(&mut self, frame: &'f mut DrawFrame<'f>) {
        let prev_alloc_seq = self.sent_alloc_seq;
        self.sent_alloc_seq = frame.alloc_seq;
        #[allow(clippy::unnecessary_cast)]
        let frame = &raw mut *frame as *mut DrawFrame<'l>;
        let missed_frame = self.shared.fresh.swap(frame, Ordering::Release);

        // The frame is non-null and thus was not consumed by the receiver
        // but it got already replaced by the newly sent frame, so we can retire it
        if missed_frame.is_null().not() {
            self.retirer.retire_early(prev_alloc_seq);
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
        let mut frame = DrawFrame::debug_dummies(&[(1.0, 2.0), (3.0, 4.0)], &mut alloc);
        sender.send(&mut frame);

        assert!(receiver.has_fresh()); // Actually fresh
        let fresh = receiver.get_fresh();
        assert_eq!(fresh.alloc_seq, 1);
        assert_eq!(fresh.cmds.len(), 1);
        assert!(matches!(fresh.cmds[0], Cmd::DrawSQuads { .. }));

        let fresh = receiver.get_fresh(); // The same as last time
        assert_eq!(fresh.alloc_seq, 1);

        let mut alloc = manager.get_frame_alloc(); // Frame 2
        let mut frame = DrawFrame::debug_dummies(&[(1.0, 2.0), (3.0, 4.0)], &mut alloc);
        sender.send(&mut frame);

        let mut alloc = manager.get_frame_alloc(); // Frame 3
        let mut frame = DrawFrame::debug_dummies(&[(1.0, 2.0), (3.0, 4.0)], &mut alloc);
        sender.send(&mut frame); // Retire early frame 2

        let fresh = receiver.get_fresh(); // Get frame 3
        assert_eq!(fresh.alloc_seq, 3);

        cleanup.cleanup();
        manager.wait_until_cleanup();
    }
}
