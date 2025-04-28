use std::{
    ops::Not,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use bang_core::draw::{DRAW_FRAME_DUMMY, DrawFrame};

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
}

#[derive(Debug)]
pub struct DrawReceiver<'l> {
    shared: &'l SharedDrawState<'l>,
    retirer: AllocRetirer<'l>,
    fresh: &'l DrawFrame<'l>,
}

pub fn new_draw_pair<'l>(
    shared: &'l mut SharedDrawState<'l>,
    retirer: AllocRetirer<'l>,
) -> (DrawSender<'l>, DrawReceiver<'l>) {
    let shared = &*shared;
    let sender = DrawSender { shared };
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
        let perennial_frame = &raw mut *frame as *mut DrawFrame<'l>;
        self.shared.fresh.swap(perennial_frame, Ordering::Release);
    }
}

impl<'l> DrawReceiver<'l> {
    pub fn get_fresh<'s>(&'s mut self) -> &'s DrawFrame<'s> {
        let freshest = self.shared.fresh.swap(null_mut(), Ordering::Acquire);
        if freshest.is_null().not() {
            let retired_seq = self.fresh.alloc_seq();
            self.fresh = unsafe { &mut *freshest };
            self.retirer.retire(retired_seq);
        }
        self.fresh
    }

    pub fn has_fresh(&self) -> bool {
        self.shared.fresh.load(Ordering::Acquire).is_null().not()
    }
}
