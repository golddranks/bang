use std::{
    collections::VecDeque,
    ops::Not,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::Duration,
};

use arena::{Arena, SharedAllocState};
use bang_core::alloc::Mem;

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

#[derive(Debug)]
#[repr(C)]
pub struct AllocCleanup<'l> {
    shared: &'l SharedAllocState,
}

impl<'l> AllocCleanup<'l> {
    pub fn cleanup(&self) {
        self.shared
            .retired_seq_up_to
            .fetch_max(usize::MAX, Ordering::Release);
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct AllocManager<'l> {
    alloc_seq: usize,
    shared: &'l SharedAllocState,
    free_pool: Vec<Arena>,
    in_use: VecDeque<Arena>,
}

pub fn make_alloc_tools<'l>(
    shared: &'l mut SharedAllocState,
) -> (AllocManager<'l>, AllocRetirer<'l>, AllocCleanup<'l>) {
    (
        AllocManager {
            alloc_seq: 0,
            shared,
            free_pool: Vec::new(),
            in_use: VecDeque::new(),
        },
        AllocRetirer { shared },
        AllocCleanup { shared },
    )
}

impl<'l> AllocManager<'l> {
    pub fn wait_until_cleanup(&mut self) {
        while self.in_use.is_empty().not() {
            thread::sleep(Duration::from_millis(1));
            self.process_retired();
        }
    }

    pub fn retire_single(&mut self, seq: usize) {
        self.in_use
            .binary_search_by_key(&seq, |alloc| alloc.alloc_seq)
            .map(|early_idx| {
                let retired = self.in_use.remove(early_idx).expect("UNREACHABLE");
                self.free_pool.push(retired);
            })
            .unwrap_or(());
    }

    fn process_retired(&mut self) {
        let retired_up_to = self.shared.retired_seq_up_to.load(Ordering::SeqCst);
        let retired_early = self.shared.retired_seq_early.swap(0, Ordering::SeqCst);
        while let Some(alloc) = self.in_use.front()
            && alloc.alloc_seq <= retired_up_to
        {
            let retired = self.in_use.pop_front().expect("UNREACHABLE");
            self.free_pool.push(retired);
        }
        if retired_early > 0 {
            self.retire_single(retired_early);
        }
    }

    pub fn get_alloc<'f>(&'f mut self) -> Mem<'f> {
        self.process_retired();
        let arena_container = self.free_pool.pop().unwrap_or_default();
        self.alloc_seq += 1;
        self.in_use.push_back(arena_container);
        let arena = self.in_use.back_mut().expect("UNREACHABLE");
        Mem {
            alloc_seq: self.alloc_seq,
            arena: arena.fresh_arena(self.alloc_seq),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_manager() {
        let mut shared = SharedAllocState::default();
        let (mut manager, retirer, cleanup) = make_alloc_tools(&mut shared);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 1);
        assert_eq!(manager.in_use.len(), 1);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 2);
        assert_eq!(manager.in_use.len(), 2);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 3);
        assert_eq!(manager.in_use.len(), 3);
        assert_eq!(manager.free_pool.len(), 0);

        retirer.retire_early(2);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 4);
        assert_eq!(manager.in_use.len(), 3);
        assert_eq!(manager.free_pool.len(), 0);

        retirer.retire_up_to(3);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 5);
        assert_eq!(manager.in_use.len(), 2);
        assert_eq!(manager.free_pool.len(), 1);

        retirer.retire_up_to(5);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 6);
        assert_eq!(manager.in_use.len(), 1);
        assert_eq!(manager.free_pool.len(), 2);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 7);
        assert_eq!(manager.in_use.len(), 2);
        assert_eq!(manager.free_pool.len(), 1);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 8);
        assert_eq!(manager.in_use.len(), 3);
        assert_eq!(manager.free_pool.len(), 0);

        retirer.retire_early(7);
        retirer.retire_up_to(8);

        let alloc = manager.get_alloc();
        assert_eq!(alloc.alloc_seq, 9);
        assert_eq!(manager.in_use.len(), 1);
        assert_eq!(manager.free_pool.len(), 2);

        cleanup.cleanup();
        manager.wait_until_cleanup();
    }
}
