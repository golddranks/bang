use std::{marker::PhantomData, mem::transmute};

mod val;
mod vec;

// Erased can't be a zero-sized type, because Vec<Erased>'s capacity returns
// usize::MAX for a zero-sized type instead of an allocated capacity.

// ErasedMax is used to be a placeholder in slide storage. It ensures that
// the allocated slices are automatically maximally aligned (to 16 bytes).
struct ErasedMax {
    _padding: u128,
}

// ErasedMin is used when returning a pointer to a slice. By being minimally
// aligned (to 1 byte), it is compatible with temporarily representing a
// pointer to any type.
struct ErasedMin {
    _padding: u8,
}

/// Represents detailed memory usage statistics for the arena allocation system.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryUsage {
    pub chunk_count: usize,
    pub total_slots: usize,
    pub used_slots: usize,
    pub vector_capacity_bytes: usize,
    pub vector_content_bytes: usize,
    pub overhead_bytes: usize,
}

impl MemoryUsage {
    pub fn total_bytes(&self) -> usize {
        self.vector_capacity_bytes + self.overhead_bytes
    }

    pub fn slot_utilization_ratio(&self) -> f64 {
        if self.total_slots == 0 {
            f64::NAN
        } else {
            self.used_slots as f64 / self.total_slots as f64
        }
    }

    pub fn memory_utilization_ratio(&self) -> f64 {
        self.vector_content_bytes as f64 / self.total_bytes() as f64
    }
}

pub struct Arena<'a> {
    alloc_seq: usize,
    vec_align_1: vec::ByAlign,
    vec_align_2: vec::ByAlign,
    vec_align_4: vec::ByAlign,
    vec_align_8: vec::ByAlign,
    vec_align_16: vec::ByAlign,
    val_align_1: val::ByAlign,
    val_align_2: val::ByAlign,
    val_align_4: val::ByAlign,
    val_align_8: val::ByAlign,
    val_align_16: val::ByAlign,
    _lifetime: PhantomData<&'a mut ErasedMax>,
}

impl<'a> Drop for Arena<'a> {
    fn drop(&mut self) {
        let aligns = [
            (1, &mut self.vec_align_1),
            (2, &mut self.vec_align_2),
            (4, &mut self.vec_align_4),
            (8, &mut self.vec_align_8),
            (16, &mut self.vec_align_16),
        ];
        for (align, by_align) in aligns {
            by_align.drop(align)
        }
    }
}

impl<'a> Arena<'a> {
    fn new() -> Self {
        Arena {
            alloc_seq: 1,
            vec_align_1: vec::ByAlign::new(),
            vec_align_2: vec::ByAlign::new(),
            vec_align_4: vec::ByAlign::new(),
            vec_align_8: vec::ByAlign::new(),
            vec_align_16: vec::ByAlign::new(),
            val_align_1: val::ByAlign::new(),
            val_align_2: val::ByAlign::new(),
            val_align_4: val::ByAlign::new(),
            val_align_8: val::ByAlign::new(),
            val_align_16: val::ByAlign::new(),
            _lifetime: PhantomData,
        }
    }

    fn memory_usage(&'a self) -> MemoryUsage {
        let mut usage = MemoryUsage::default();

        let aligns = [
            (1, &self.vec_align_1),
            (2, &self.vec_align_2),
            (4, &self.vec_align_4),
            (8, &self.vec_align_8),
            (16, &self.vec_align_16),
        ];

        usage.overhead_bytes += size_of::<Arena>();

        for (align, by_align) in aligns {
            by_align.memory_usage(align, self.alloc_seq, &mut usage);
        }

        usage
    }

    const fn get_vec_align(&mut self, align: usize) -> &mut vec::ByAlign {
        match align {
            1 => &mut self.vec_align_1,
            2 => &mut self.vec_align_2,
            4 => &mut self.vec_align_4,
            8 => &mut self.vec_align_8,
            16 => &mut self.vec_align_16,
            _ => panic!("Unsupported alignment"),
        }
    }

    const fn get_val_align(&mut self, align: usize) -> &mut val::ByAlign {
        match align {
            1 => &mut self.val_align_1,
            2 => &mut self.val_align_2,
            4 => &mut self.val_align_4,
            8 => &mut self.val_align_8,
            16 => &mut self.val_align_16,
            _ => panic!("Unsupported alignment"),
        }
    }

    pub fn allocate_val<T>(&mut self, val: T) -> &'a T {
        let byte_size = size_of::<T>();
        let by_align = self.get_val_align(align_of::<T>());
        let erased = by_align.allocate_val(byte_size);
        let typed_ptr = erased as *mut ErasedMin as *mut T;
        unsafe {
            typed_ptr.write(val);
            &mut *typed_ptr
        }
    }

    pub fn allocate_vec<T>(&mut self) -> &'a mut Vec<T> {
        let n_size = const { size_of::<T>() / align_of::<T>() };
        let seq = self.alloc_seq;
        let by_align = self.get_vec_align(align_of::<T>());
        let erased = by_align.allocate_vec(n_size, seq);
        let typed_ptr = erased as *mut Vec<ErasedMax> as *mut Vec<T>;

        // Safety: This cast is safe because:
        // 1. Vec<T> and Vec<Erased> have identical memory layouts regardless of T
        // 2. We immediately set the length to 0, so no values of the previous type
        //    will be incorrectly interpreted as type T
        let typed = unsafe { &mut *typed_ptr };

        // Safety: Setting len=0 is safe even when reusing a vector that previously held
        // values of a different type because:
        // 1. Setting the length to 0 can't overflow the capacity
        // 2. We deliberately avoid running destructors on any potential existing elements
        //    (which would be unsound as they might be of a different type)
        // 3. This is valid in Rust - not calling destructors is allowed
        // 4. Any memory previously used by the vector becomes inaccessible but remains
        //    allocated, allowing us to reuse the capacity
        unsafe { typed.set_len(0) };
        typed
    }

    fn reset(&mut self, seq: usize) {
        self.alloc_seq = seq;
        self.val_align_4.reset();
    }
}

/// Test: Allocated things can't outlive the allocator
/// ```compile_fail
/// let a = {
///     let mut alloc = Allocator::new();
///     let mut arena = alloc.new_arena();
///     let arena = &mut arena;
///
///     let vec1 = arena.allocate_vec::<u32>();
///     vec1.push(42);
///     assert_eq!(vec1.len(), 1);
///     vec1
/// };
/// ```
///
/// Test: getting a new arena makes the old one unusable
/// ```compile_fail
/// let mut alloc = Allocator::new();
/// let mut arena = alloc.new_arena();
///
/// let vec1 = arena.allocate_vec::<u32>();
/// vec1.push(42);
/// assert_eq!(vec1.len(), 1);
///
/// let mut arena = alloc.new_arena();
/// vec1.push(43);
/// ```
pub struct Allocator {
    alloc_seq: usize,
    arena: Arena<'static>,
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            alloc_seq: 0,
            arena: Arena::new(),
        }
    }

    /// ```
    /// use vec_arena::Allocator;
    /// let mut alloc = Allocator::new();
    /// let _ = alloc.memory_usage();
    /// let arena = alloc.new_arena();
    /// let vec1 = arena.allocate_vec::<u32>();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// ```
    ///
    /// ```compile_fail
    /// use vec_arena::Allocator;
    /// let mut alloc = Allocator::new();
    /// let arena = alloc.new_arena();
    /// let _ = alloc.memory_usage();
    /// let vec1 = arena.allocate_vec::<u32>();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// ```
    ///
    /// ```compile_fail
    /// use vec_arena::Allocator;
    /// let mut alloc = Allocator::new();
    /// let arena = alloc.new_arena();
    /// let vec1 = arena.allocate_vec::<u32>();
    /// let _ = alloc.memory_usage();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// ```
    ///
    /// ```
    /// use vec_arena::Allocator;
    /// let mut alloc = Allocator::new();
    /// let arena = alloc.new_arena();
    /// let vec1 = arena.allocate_vec::<u32>();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// let _ = alloc.memory_usage();
    /// let mut alloc = Allocator::new();
    /// ```
    pub fn memory_usage(&self) -> MemoryUsage {
        self.arena.memory_usage()
    }

    pub fn new_arena<'a>(&'a mut self) -> &'a mut Arena<'a> {
        self.alloc_seq += 1;
        self.arena.reset(self.alloc_seq);
        // Safety: This lifetime transmutation is safe because:
        // 1. We are only changing the lifetime parameter, not the type structure
        // 2. The arena's 'static lifetime is being shortened to match self's lifetime 'a
        // 3. This ensures the arena cannot be used beyond the lifetime of self
        // 4. The Rust borrow checker then prevents accessing the returned Vec references
        //    after a new arena is created (as demonstrated by compile_fail tests)
        unsafe { transmute(&mut self.arena) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_allocation() {
        let mut arena = Arena::new();
        let _vec = arena.allocate_vec::<u32>();

        drop(arena);
    }

    #[test]
    fn test_empty_allocation() {
        let arena = Arena::new();

        drop(arena);
    }

    #[test]
    fn test_basic_allocation() {
        let mut arena = Arena::new();

        let vec = arena.allocate_vec::<u32>();
        assert!(vec.is_empty());

        vec.push(42);
        vec.push(100);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 42);
        assert_eq!(vec[1], 100);

        drop(arena);
    }

    #[test]
    fn test_same_layout() {
        let mut arena = Arena::new();

        let vec1 = arena.allocate_vec::<u8>();
        let vec2 = arena.allocate_vec::<u8>();

        vec1.extend_from_slice(&[1, 2, 3]);
        vec2.extend_from_slice(&[4, 5, 6]);

        assert_eq!(vec1, &[1, 2, 3]);
        assert_eq!(vec2, &[4, 5, 6]);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut arena = Arena::new();

        let vec1 = arena.allocate_vec::<u8>();
        let vec2 = arena.allocate_vec::<u16>();
        let vec3 = arena.allocate_vec::<u32>();
        let vec4 = arena.allocate_vec::<u64>();
        let vec5 = arena.allocate_vec::<u8>();

        vec1.extend_from_slice(&[1, 2, 3]);
        vec2.extend_from_slice(&[10, 20, 30]);
        vec3.extend_from_slice(&[100, 200, 300]);
        vec4.extend_from_slice(&[1000, 2000, 3000]);
        vec5.extend_from_slice(&[4, 5, 6]);

        assert_eq!(vec1, &[1, 2, 3]);
        assert_eq!(vec2, &[10, 20, 30]);
        assert_eq!(vec3, &[100, 200, 300]);
        assert_eq!(vec4, &[1000, 2000, 3000]);
        assert_eq!(vec5, &[4, 5, 6]);
    }

    #[test]
    fn test_reset() {
        let mut alloc = Allocator::default();
        let arena = alloc.new_arena();

        let vec = arena.allocate_vec::<u32>();
        vec.push(42);
        assert_eq!(vec.len(), 1);

        let arena = alloc.new_arena();
        let vec = arena.allocate_vec::<u32>();
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_different_alignments() {
        let mut arena = Arena::new();

        let vec1 = arena.allocate_vec::<u8>();
        let vec2 = arena.allocate_vec::<u16>();
        let vec4 = arena.allocate_vec::<u32>();
        let vec8 = arena.allocate_vec::<u64>();
        let vec16 = arena.allocate_vec::<u128>();
        let vec8_8 = arena.allocate_vec::<[u64; 2]>();

        vec1.push(1);
        vec2.push(2);
        vec4.push(4);
        vec8.push(8);
        vec16.push(16);
        vec8_8.push([16, 16]);

        assert_eq!(vec1[0], 1);
        assert_eq!(vec2[0], 2);
        assert_eq!(vec4[0], 4);
        assert_eq!(vec8[0], 8);
        assert_eq!(vec16[0], 16);
        assert_eq!(vec8_8[0], [16, 16]);
    }

    #[test]
    fn test_large_allocation() {
        let mut arena = Arena::new();

        let vec = arena.allocate_vec::<u32>();
        for i in 0..1000 {
            vec.push(i);
        }

        assert_eq!(vec.len(), 1000);
        for i in 0..1000 {
            assert_eq!(vec[i], i as u32);
        }
    }

    #[test]
    fn test_multiple_vectors_same_type() {
        let mut arena = Arena::new();

        let vectors: Vec<_> = (0..10)
            .map(|i| {
                let vec = arena.allocate_vec::<u32>();
                vec.push(i);
                vec
            })
            .collect();

        for (i, vec) in vectors.iter().enumerate() {
            assert_eq!(vec.len(), 1);
            assert_eq!(vec[0], i as u32);
        }
    }

    #[test]
    fn test_vector_capacity_reuse() {
        let mut alloc = Allocator::default();

        let arena = alloc.new_arena();

        // Grow u32 vecs (4-byte alignment)
        let vec_u32_a = arena.allocate_vec::<u32>();
        let vec_u32_b = arena.allocate_vec::<u32>();
        for i in 0..1000 {
            vec_u32_a.push(i);
        }
        for i in 0..500 {
            vec_u32_b.push(i);
        }

        let arena = alloc.new_arena();
        let vec_u32 = arena.allocate_vec::<u32>();
        let vec_f32 = arena.allocate_vec::<f32>(); // f32 also has 4-byte alignment
        assert!(
            vec_u32.capacity() >= 1000,
            "u32 vector capacity should be reused"
        );
        assert!(
            vec_f32.capacity() >= 500,
            "Vectors with same alignment should share capacity pool"
        );
    }

    #[test]
    fn test_zero_sized() {
        let mut alloc = Allocator::default();
        let arena = alloc.new_arena();

        let vec = arena.allocate_vec::<()>();

        assert_eq!(vec.capacity(), usize::MAX);
        assert_eq!(vec.len(), 0);

        vec.push(());

        assert_eq!(vec.capacity(), usize::MAX);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    #[should_panic]
    fn test_unsupported_alignment() {
        let mut alloc = Allocator::default();
        let arena = alloc.new_arena();

        #[repr(align(32))]
        struct Test;

        let _ = arena.allocate_vec::<Test>();
    }

    #[test]
    fn test_memory_usage() {
        let mut alloc = Allocator::default();
        let arena = alloc.new_arena();

        let initial_usage = arena.memory_usage();
        assert_eq!(initial_usage.used_slots, 0);
        assert_eq!(initial_usage.chunk_count, 0);
        assert!(initial_usage.slot_utilization_ratio().is_nan());

        let vec1 = arena.allocate_vec::<u8>();
        vec1.extend_from_slice(&[1, 2, 3, 4, 5]);
        let _vec1_2 = arena.allocate_vec::<u8>();
        let _vec2 = arena.allocate_vec::<u32>();
        let _vec3 = arena.allocate_vec::<u64>();
        let _vec3_2 = arena.allocate_vec::<(u64, u64)>();

        let after_alloc = arena.memory_usage();
        assert_eq!(after_alloc.used_slots, 5);
        assert_eq!(after_alloc.chunk_count, 4);

        let arena = alloc.new_arena();
        let vec1 = arena.allocate_vec::<u8>();
        let vec2 = arena.allocate_vec::<u32>();
        let vec3 = arena.allocate_vec::<u64>();

        vec1.extend_from_slice(&[1, 2, 3, 4, 5]);
        vec2.extend_from_slice(&[10, 20, 30]);
        vec3.extend_from_slice(&[100, 200]);

        let after_data = arena.memory_usage();

        let expected_content_bytes = 5 * std::mem::size_of::<u8>()
            + 3 * std::mem::size_of::<u32>()
            + 2 * std::mem::size_of::<u64>();

        assert_eq!(after_data.vector_content_bytes, expected_content_bytes);
        assert!(after_data.vector_capacity_bytes >= expected_content_bytes);
        assert!(after_data.overhead_bytes > 0);
        assert!(after_data.memory_utilization_ratio() > 0.0);
        assert!(after_data.slot_utilization_ratio() > 0.0);

        let arena = alloc.new_arena();
        let after_reset = arena.memory_usage();
        assert!(after_reset.slot_utilization_ratio() == 0.0);

        assert_eq!(
            after_reset.vector_content_bytes, 0,
            "Vector content bytes should be 0 after reset"
        );
        assert!(
            after_reset.vector_capacity_bytes > 0,
            "Vector capacity bytes should be preserved"
        );
        assert!(
            after_reset.total_slots > 0,
            "Total slots should be preserved after reset"
        );
        assert_eq!(
            after_reset.used_slots, 0,
            "Used slots should be 0 after reset"
        );
        assert!(
            after_reset.overhead_bytes > 0,
            "Overhead bytes should exist after reset"
        );

        for _ in 0..10 {
            let vec = arena.allocate_vec::<u32>();
            for _ in 0..100 {
                vec.push(42);
            }
        }

        let alloc_usage = alloc.memory_usage();
        assert_eq!(alloc_usage.used_slots, 10);
        assert_eq!(
            alloc_usage.vector_content_bytes,
            10 * 100 * std::mem::size_of::<u32>()
        );
        assert!(alloc_usage.chunk_count > 0);
    }

    #[test]
    fn test_chunk_consolidation() {
        let mut alloc = Allocator::default();

        // First arena: allocate vectors to force multiple chunks
        let arena = alloc.new_arena();

        let mut original_addresses_u32 = Vec::new();
        let mut original_addresses_u64 = Vec::new();

        for _ in 0..100 {
            let vec_u32 = arena.allocate_vec::<u32>();
            let vec_u64 = arena.allocate_vec::<u64>();

            original_addresses_u32.push(vec_u32 as *mut _);
            original_addresses_u64.push(vec_u64 as *mut _);

            // Force capacity growth
            for i in 0..20 {
                vec_u32.push(i);
                vec_u64.push(i as u64);
            }
        }

        // First consolidation of the chunks
        let arena = alloc.new_arena();

        let mut first_addresses_u32 = Vec::new();
        let mut first_addresses_u64 = Vec::new();

        for _ in 0..100 {
            let vec_u32 = arena.allocate_vec::<u32>();
            let vec_u64 = arena.allocate_vec::<u64>();

            // Verify capacities are preserved
            assert!(vec_u32.capacity() >= 20);
            assert!(vec_u64.capacity() >= 20);

            first_addresses_u32.push(vec_u32 as *mut _);
            first_addresses_u64.push(vec_u64 as *mut _);
        }

        for i in 0..100 {
            assert_ne!(
                original_addresses_u32[i], first_addresses_u32[i],
                "u32 vector addresses should change after first consolidation"
            );
            assert_ne!(
                original_addresses_u64[i], first_addresses_u64[i],
                "u64 vector addresses should change after first consolidation"
            );
        }

        // Second consolidation
        let arena = alloc.new_arena();

        // Verify addresses remain stable after second consolidation
        for i in 0..100 {
            let vec_u32 = arena.allocate_vec::<u32>();
            let vec_u64 = arena.allocate_vec::<u64>();

            // Verify capacities are still preserved
            assert!(vec_u32.capacity() >= 20);
            assert!(vec_u64.capacity() >= 20);

            assert_eq!(
                vec_u32 as *mut _, first_addresses_u32[i],
                "u32 vector addresses should be stable after second consolidation"
            );
            assert_eq!(
                vec_u64 as *mut _, first_addresses_u64[i],
                "u64 vector addresses should be stable after second consolidation"
            );
        }
    }
}
