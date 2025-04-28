use std::{
    collections::VecDeque,
    mem::transmute,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
#[repr(C)]
pub struct SharedAllocState {
    retired_seq: AtomicUsize,
}

impl Default for SharedAllocState {
    fn default() -> Self {
        Self {
            retired_seq: AtomicUsize::new(0),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct AllocRetirer<'l> {
    shared: &'l SharedAllocState,
}

impl<'l> AllocRetirer<'l> {
    pub fn retire(&self, seq: usize) {
        self.shared.retired_seq.fetch_max(seq, Ordering::Release);
    }
}
use bang_core::alloc::Alloc;

#[derive(Debug)]
#[repr(C)]
pub struct AllocManager<'l> {
    alloc_seq: usize,
    shared: &'l SharedAllocState,
    free_pool: Vec<Alloc<'static>>,
    in_use: VecDeque<Alloc<'static>>,
}

pub fn new_alloc_pair<'l>(
    shared: &'l mut SharedAllocState,
) -> (AllocManager<'l>, AllocRetirer<'l>) {
    (
        AllocManager {
            alloc_seq: 0,
            shared,
            free_pool: Vec::new(),
            in_use: VecDeque::new(),
        },
        AllocRetirer { shared },
    )
}

impl<'l> AllocManager<'l> {
    fn process_retired(&mut self) {
        let latest_retired = self.shared.retired_seq.swap(0, Ordering::Acquire);
        while let Some(alloc) = self.in_use.front()
            && alloc.alloc_seq <= latest_retired
        {
            let retired = self.in_use.pop_front().expect("UNREACHABLE");
            self.free_pool.push(retired);
        }
    }

    pub fn get_frame_alloc<'f, 's>(&'s mut self) -> &'s mut Alloc<'f> {
        self.process_retired();
        let mut alloc = self.free_pool.pop().unwrap_or_default();
        self.alloc_seq += 1;
        alloc.reset(self.alloc_seq);
        self.in_use.push_back(alloc);
        let alloc_mut = self.in_use.back_mut().expect("UNREACHABLE");
        unsafe { transmute::<&mut Alloc<'static>, &mut Alloc<'f>>(alloc_mut) }
    }
}
