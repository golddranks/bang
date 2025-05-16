use std::{
    fmt::Debug,
    mem::{needs_drop, transmute},
    sync::atomic::{AtomicU16, AtomicU64, Ordering},
};

mod guard;
//mod managed;
mod managed_sync;
#[cfg(any(test, doctest))]
mod tests;
mod val;
mod vec;

pub use guard::{ArenaGuard, Sink};
//pub use managed::{Id, Managed};
pub use managed_sync::{Id, Managed};

// Placeholder types for type erasure
//
// Erased can't be a zero-sized type, because Vec<_>'s capacity returns
// usize::MAX for a zero-sized type instead of an allocated capacity.

/// ErasedMax is used to be a placeholder in memory storage. It ensures that
/// the allocated slices are automatically maximally aligned (to 16 bytes).
struct ErasedMax {
    _padding: u128,
}

/// Erased is used when returning a pointer to a slice. By being minimally
/// aligned (to 1 byte), it is compatible with temporarily representing a
/// pointer to any type.
pub struct Erased {
    _padding: u8,
}

// Memory usage reporting

/// MemoryUsage represents detailed memory usage report for the arena allocation.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryUsage {
    pub capacity_bytes: usize,
    pub content_bytes: usize,
    pub overhead_bytes: usize,
}

impl MemoryUsage {
    pub fn total_bytes(&self) -> usize {
        self.capacity_bytes + self.overhead_bytes
    }

    pub fn memory_utilization_ratio(&self) -> f64 {
        self.content_bytes as f64 / self.total_bytes() as f64
    }
}

// Main arena wrapper

#[derive(Debug)]
pub struct Arena {
    pub alloc_seq: usize,
    guard: ArenaGuard<'static>,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            alloc_seq: 0,
            guard: ArenaGuard::new(),
        }
    }
}

impl Arena {
    pub fn memory_usage(&mut self) -> MemoryUsage {
        unsafe { self.guard.memory_usage() }
    }

    /// Gets ArenaGuard that has the lifetime equivalent to the borrowed self
    /// instead of 'static, which is a placeholder when the guard is not in use.
    fn guard_with_short_lifetime<'a>(&'a mut self) -> &'a mut ArenaGuard<'a> {
        // Safety: This lifetime transmutation is safe because:
        // 1. We are only changing the lifetime parameter, not the type structure
        // 2. The arena's 'static lifetime is being shortened to match self's lifetime 'a
        // 3. This ensures the arena cannot be used beyond the lifetime of self
        // 4. The Rust borrow checker then prevents accessing the returned Vec references
        //    after a new arena is created (as demonstrated by compile_fail tests)
        unsafe { transmute(&mut self.guard) }
    }

    pub fn fresh_arena<'a>(&'a mut self, seq: usize) -> &'a mut ArenaGuard<'a> {
        self.alloc_seq = seq;
        unsafe { self.guard.reset(seq) };
        self.guard_with_short_lifetime()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SharedAllocState {
    pub managed_arena_seq: AtomicU16,
    pub alloc_seq: AtomicU64,
    pub retired_seq_up_to: AtomicU64,
    pub retired_seq_early: AtomicU64,
}

impl Default for SharedAllocState {
    fn default() -> Self {
        Self {
            managed_arena_seq: AtomicU16::new(0),
            alloc_seq: AtomicU64::new(0),
            retired_seq_up_to: AtomicU64::new(0),
            retired_seq_early: AtomicU64::new(0),
        }
    }
}

impl SharedAllocState {
    fn current_seq(&self) -> u64 {
        self.alloc_seq.load(Ordering::Acquire)
    }
}

const fn check_alignment_static<T>() {
    // Panics: Static assert, so no runtime panics are possible
    const {
        let align = align_of::<T>();
        assert!(align.is_power_of_two(), "Align must be power of two!");
        assert!(align >= 1, "Align must be at least 1");
        assert!(align <= 16, "Align must be at most 16");
    };
}

const fn check_drop_static<T>() {
    // Panics: Static assert, so no runtime panics are possible
    const { assert!(!needs_drop::<T>(), "Can't allocate type with Drop!") };
}

const fn check_string_vec_layout_static() {
    // Panics: Static assert, so no runtime panics are possible
    // The field offset layout is not possible to check with
    // compile-time assertions, but it is additionally checked in tests
    const {
        assert!(size_of::<String>() == size_of::<Vec<u8>>());
        assert!(align_of::<String>() == align_of::<Vec<u8>>())
    }
}
