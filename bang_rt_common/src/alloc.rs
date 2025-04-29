use std::{
    collections::VecDeque,
    mem::transmute,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
#[repr(C)]
pub struct SharedAllocState {
    retired_seq_up_to: AtomicUsize,
    retired_seq_early: AtomicUsize,
}

impl Default for SharedAllocState {
    fn default() -> Self {
        Self {
            retired_seq_up_to: AtomicUsize::new(0),
            retired_seq_early: AtomicUsize::new(0),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct AllocRetirer<'l> {
    shared: &'l SharedAllocState,
}

impl<'l> AllocRetirer<'l> {
    pub fn retire_up_to(&self, seq: usize) {
        self.shared
            .retired_seq_up_to
            .fetch_max(seq, Ordering::Release);
    }

    pub fn retire_early(&self, seq: usize) {
        let _ = self.shared.retired_seq_early.compare_exchange(
            0,
            seq,
            Ordering::Release,
            Ordering::Relaxed,
        );
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
        let retired_up_to = self.shared.retired_seq_up_to.swap(0, Ordering::Acquire);
        let retired_early = self.shared.retired_seq_early.swap(0, Ordering::Acquire);
        while let Some(alloc) = self.in_use.front()
            && alloc.alloc_seq <= retired_up_to
        {
            let retired = self.in_use.pop_front().expect("UNREACHABLE");
            self.free_pool.push(retired);
        }
        if retired_early > 0
            && let Ok(early_idx) = self
                .in_use
                .binary_search_by_key(&retired_early, |alloc| alloc.alloc_seq)
        {
            let retired = self.in_use.remove(early_idx).expect("UNREACHABLE");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_manager() {
        let mut shared = SharedAllocState::default();
        let (mut manager, retirer) = new_alloc_pair(&mut shared);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 1);
        assert_eq!(manager.in_use.len(), 1);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 2);
        assert_eq!(manager.in_use.len(), 2);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 3);
        assert_eq!(manager.in_use.len(), 3);
        assert_eq!(manager.free_pool.len(), 0);

        retirer.retire_early(2);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 4);
        assert_eq!(manager.in_use.len(), 3);
        assert_eq!(manager.free_pool.len(), 0);

        retirer.retire_up_to(3);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 5);
        assert_eq!(manager.in_use.len(), 2);
        assert_eq!(manager.free_pool.len(), 1);

        retirer.retire_up_to(5);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 6);
        assert_eq!(manager.in_use.len(), 1);
        assert_eq!(manager.free_pool.len(), 2);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 7);
        assert_eq!(manager.in_use.len(), 2);
        assert_eq!(manager.free_pool.len(), 1);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 8);
        assert_eq!(manager.in_use.len(), 3);
        assert_eq!(manager.free_pool.len(), 0);

        retirer.retire_early(7);
        retirer.retire_up_to(8);

        let alloc = manager.get_frame_alloc();
        assert_eq!(alloc.alloc_seq, 9);
        assert_eq!(manager.in_use.len(), 1);
        assert_eq!(manager.free_pool.len(), 2);
    }
}
