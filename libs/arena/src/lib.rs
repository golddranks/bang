use std::{fmt::Debug, mem::transmute};

pub use guard::ArenaGuard;
pub use managed::{Id, Managed};

mod guard;
mod managed;
#[cfg(any(test, doctest))]
mod tests;
mod val;
mod vec;

// Placeholder types for type erasure
//
// Erased can't be a zero-sized type, because Vec<_>'s capacity returns
// usize::MAX for a zero-sized type instead of an allocated capacity.

/// ErasedMax is used to be a placeholder in memory storage. It ensures that
/// the allocated slices are automatically maximally aligned (to 16 bytes).
struct ErasedMax {
    _padding: u128,
}

/// ErasedMin is used when returning a pointer to a slice. By being minimally
/// aligned (to 1 byte), it is compatible with temporarily representing a
/// pointer to any type.
struct ErasedMin {
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
    arena: ArenaGuard<'static>,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            alloc_seq: 0,
            arena: ArenaGuard::new(),
        }
    }
}

impl Arena {
    pub fn memory_usage(&mut self) -> MemoryUsage {
        unsafe { self.arena.memory_usage() }
    }

    pub fn fresh_arena<'a>(&'a mut self, seq: usize) -> &'a mut ArenaGuard<'a> {
        self.alloc_seq = seq;
        unsafe { self.arena.reset(seq) };
        // Safety: This lifetime transmutation is safe because:
        // 1. We are only changing the lifetime parameter, not the type structure
        // 2. The arena's 'static lifetime is being shortened to match self's lifetime 'a
        // 3. This ensures the arena cannot be used beyond the lifetime of self
        // 4. The Rust borrow checker then prevents accessing the returned Vec references
        //    after a new arena is created (as demonstrated by compile_fail tests)
        unsafe { transmute(&mut self.arena) }
    }
}
