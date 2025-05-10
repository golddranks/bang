use std::{
    marker::PhantomData,
    mem::{needs_drop, transmute},
};

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

#[derive(Debug)]
pub struct Arena<'a> {
    pub alloc_seq: usize,
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
        // We need to manually drop the vecs to avoid memory leaks.
        // The vecs are not automatically dropped because they are stored as
        // type-erased vecs in MaybeUninit<_>. The destructors of the contents
        // of the vecs and single values are not called; this is part of the
        // library API contract.
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

    fn memory_usage(&self) -> MemoryUsage {
        let mut usage = MemoryUsage::default();

        usage.overhead_bytes += size_of::<Arena>();

        let vec_aligns = [
            (1, &self.vec_align_1),
            (2, &self.vec_align_2),
            (4, &self.vec_align_4),
            (8, &self.vec_align_8),
            (16, &self.vec_align_16),
        ];

        for (align, by_align) in vec_aligns {
            by_align.memory_usage(align, self.alloc_seq, &mut usage);
        }

        let val_aligns = [
            &self.val_align_1,
            &self.val_align_2,
            &self.val_align_4,
            &self.val_align_8,
            &self.val_align_16,
        ];

        for by_align in val_aligns {
            by_align.memory_usage(&mut usage);
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

    pub fn allocate_vec<T>(&mut self) -> &'a mut Vec<T> {
        const { assert!(!needs_drop::<T>()) };
        self.allocate_vec_ignore_drop()
    }

    pub fn allocate_val<T>(&mut self, val: T) -> &'a mut T {
        const { assert!(!needs_drop::<T>()) };
        self.allocate_val_ignore_drop(val)
    }

    pub fn allocate_val_ignore_drop<T>(&mut self, val: T) -> &'a mut T {
        let byte_size = size_of::<T>();
        let by_align = self.get_val_align(align_of::<T>());
        let erased_ptr = by_align.allocate_val(byte_size);
        let typed_ptr = erased_ptr as *mut T;
        // Safety: erased_ptr has size and alignment that is ensured to be
        // compatible with T. The memory is originally stored as MaybeUninit<_>,
        // and is not touched by any other route as long as the borrow is live.
        unsafe {
            typed_ptr.write(val);
            &mut *typed_ptr
        }
    }

    pub fn allocate_vec_ignore_drop<T>(&mut self) -> &'a mut Vec<T> {
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
        self.val_align_1.reset();
        self.val_align_2.reset();
        self.val_align_4.reset();
        self.val_align_8.reset();
        self.val_align_16.reset();
    }
}

/// Test: Allocated things can't outlive the allocator
/// ```compile_fail
/// let a = {
///     let mut alloc = Allocator::new();
///     let mut arena = alloc.new_arena(0);
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
/// let mut arena = alloc.new_arena(0);
///
/// let vec1 = arena.allocate_vec::<u32>();
/// vec1.push(42);
/// assert_eq!(vec1.len(), 1);
///
/// let mut arena = alloc.new_arena(1);
/// vec1.push(43);
/// ```
#[derive(Debug)]
pub struct ArenaContainer {
    pub alloc_seq: usize,
    arena: Arena<'static>,
}

impl Default for ArenaContainer {
    fn default() -> Self {
        Self {
            alloc_seq: 0,
            arena: Arena::new(),
        }
    }
}

impl ArenaContainer {
    /// ```
    /// use arena::ArenaContainer;
    /// let mut alloc = ArenaContainer::default();
    /// let _ = alloc.memory_usage();
    /// let arena = alloc.new_arena(0);
    /// let vec1 = arena.allocate_vec::<u32>();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// ```
    ///
    /// ```compile_fail
    /// use arena::ArenaContainer;
    /// let mut alloc = ArenaContainer::default();
    /// let arena = alloc.new_arena(0);
    /// let _ = alloc.memory_usage();
    /// let vec1 = arena.allocate_vec::<u32>();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// ```
    ///
    /// ```compile_fail
    /// use arena::ArenaContainer;
    /// let mut alloc = ArenaContainer::default();
    /// let arena = alloc.new_arena(0);
    /// let vec1 = arena.allocate_vec::<u32>();
    /// let _ = alloc.memory_usage();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// ```
    ///
    /// ```
    /// use arena::ArenaContainer;
    /// let mut alloc = ArenaContainer::default();
    /// let arena = alloc.new_arena(0);
    /// let vec1 = arena.allocate_vec::<u32>();
    ///
    /// vec1.push(42);
    /// assert_eq!(vec1.len(), 1);
    /// let _ = alloc.memory_usage();
    /// ```
    pub fn memory_usage(&self) -> MemoryUsage {
        self.arena.memory_usage()
    }

    pub fn new_arena<'a>(&'a mut self, seq: usize) -> &'a mut Arena<'a> {
        self.alloc_seq = seq;
        self.arena.reset(seq);
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
    use std::ptr::addr_eq;

    #[test]
    fn test_allocator() {}

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
    fn test_basic_value_allocation() {
        let mut arena = Arena::new();

        // Allocate and check values
        let val1 = arena.allocate_val(42u32);
        let val2 = arena.allocate_val(100u32);

        assert_eq!(*val1, 42);
        assert_eq!(*val2, 100);

        // Update value
        *val1 = 99;
        assert_eq!(*val1, 99);
        assert_eq!(*val2, 100);

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
        let mut alloc = ArenaContainer::default();
        let arena = alloc.new_arena(0);

        let vec = arena.allocate_vec::<u32>();
        vec.push(42);
        assert_eq!(vec.len(), 1);

        let arena = alloc.new_arena(1);
        let vec = arena.allocate_vec::<u32>();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_value_reset() {
        let mut alloc = ArenaContainer::default();
        let arena = alloc.new_arena(0);

        let val1 = arena.allocate_val(42u32);
        let val2 = arena.allocate_val(43u64);
        assert_eq!(*val1, 42u32);
        assert_eq!(*val2, 43u64);

        // Get a new arena, which should reset the previous allocations
        let arena = alloc.new_arena(1);

        // New allocations should work and potentially reuse memory
        let new_val1 = arena.allocate_val(100u32);
        let new_val2 = arena.allocate_val(200u64);
        assert_eq!(*new_val1, 100u32);
        assert_eq!(*new_val2, 200u64);
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

        for i in 0..1000 {
            assert_eq!(vec[i], i as u32);
        }

        drop(arena);
    }

    #[test]
    fn test_large_value_allocation() {
        let mut arena = Arena::new();

        // Allocate a large array
        let large_value = arena.allocate_val([42u64; 32]); // 256 bytes

        // Verify it worked
        assert_eq!(large_value[0], 42);
        assert_eq!(large_value[31], 42);

        // Modify part of it
        large_value[10] = 100;
        large_value[20] = 200;

        assert_eq!(large_value[10], 100);
        assert_eq!(large_value[20], 200);

        // Allocate another large value and verify they don't interfere
        let large_value2 = arena.allocate_val([84u64; 32]);

        // Verify that values in the first array are intact after second allocation
        assert_eq!(large_value[10], 100);
        assert_eq!(large_value[20], 200);
        assert_eq!(large_value2[10], 84);

        // Modify the second array and verify the first array remains unchanged
        large_value2[10] = 1010;
        large_value2[20] = 2020;

        // Verify both arrays have their own expected values (non-interference)
        assert_eq!(large_value[10], 100);
        assert_eq!(large_value[20], 200);
        assert_eq!(large_value2[10], 1010);
        assert_eq!(large_value2[20], 2020);

        // Check memory usage
        let usage = arena.memory_usage();
        let expected_content = 2 * 32 * std::mem::size_of::<u64>(); // 2 arrays of 32 u64s
        assert_eq!(usage.content_bytes, expected_content);

        drop(arena);
    }

    #[test]
    fn test_value_different_alignments() {
        let mut arena = Arena::new();

        // Test different alignment requirements
        let val1 = arena.allocate_val(0x42u8); // align 1
        let val2 = arena.allocate_val(0x4243u16); // align 2
        let val3 = arena.allocate_val(0x42434445u32); // align 4
        let val4 = arena.allocate_val(0x4243444546474849u64); // align 8

        // Verify values were stored correctly
        assert_eq!(*val1, 0x42u8);
        assert_eq!(*val2, 0x4243u16);
        assert_eq!(*val3, 0x42434445u32);
        assert_eq!(*val4, 0x4243444546474849u64);

        // Verify exact memory usage
        let usage = arena.memory_usage();
        let expected_content = std::mem::size_of::<u8>()
            + std::mem::size_of::<u16>()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<u64>();
        assert_eq!(usage.content_bytes, expected_content);

        // Modify the values and check they're independent
        *val1 = 0x11;
        *val2 = 0x2222;
        *val3 = 0x33333333;
        *val4 = 0x4444444444444444;

        assert_eq!(*val1, 0x11);
        assert_eq!(*val2, 0x2222);
        assert_eq!(*val3, 0x33333333);
        assert_eq!(*val4, 0x4444444444444444);

        // Check exact content bytes
        let align_usage = arena.memory_usage();
        assert_eq!(
            align_usage.content_bytes,
            std::mem::size_of::<u8>()
                + std::mem::size_of::<u16>()
                + std::mem::size_of::<u32>()
                + std::mem::size_of::<u64>()
        );
    }

    #[test]
    fn test_mixed_vec_and_val_allocation() {
        let mut arena = Arena::new();

        // Allocate vectors and values interleaved
        let vec1 = arena.allocate_vec::<u32>();
        let val1 = arena.allocate_val(42u64);
        let vec2 = arena.allocate_vec::<u8>();
        let val2 = arena.allocate_val(43u16);

        // Use the vectors
        vec1.push(100);
        vec1.push(200);
        vec2.push(10);
        vec2.push(20);
        vec2.push(30);

        // Check everything works correctly
        assert_eq!(vec1.len(), 2);
        assert_eq!(vec1[0], 100);
        assert_eq!(vec1[1], 200);

        assert_eq!(vec2.len(), 3);
        assert_eq!(vec2[0], 10);
        assert_eq!(vec2[1], 20);
        assert_eq!(vec2[2], 30);

        assert_eq!(*val1, 42u64);
        assert_eq!(*val2, 43u16);

        // Verify exact memory usage
        let usage = arena.memory_usage();
        let expected_content = std::mem::size_of::<u64>()
            + std::mem::size_of::<u16>()
            + (2 * std::mem::size_of::<u32>())
            + (3 * std::mem::size_of::<u8>());
        assert_eq!(usage.content_bytes, expected_content);

        // Modify values
        *val1 = 99;
        *val2 = 999;

        assert_eq!(*val1, 99u64);
        assert_eq!(*val2, 999u16);
    }

    #[test]
    fn test_zero_sized_values() {
        let mut arena = Arena::new();

        let empty1 = arena.allocate_val(());
        let empty2 = arena.allocate_val(());

        assert!(addr_eq(empty1, empty2));
    }

    #[test]
    fn test_value_memory_usage() {
        let mut arena = Arena::new();

        // Check initial memory usage
        let empty_usage = arena.memory_usage();
        assert_eq!(empty_usage.content_bytes, 0);
        assert_eq!(empty_usage.capacity_bytes, 80);

        // Allocate small values
        let _val1 = arena.allocate_val(1u8);
        let _val2 = arena.allocate_val(2u16);
        let _val3 = arena.allocate_val(3u32);

        // Check memory usage after small allocations
        let small_alloc_usage = arena.memory_usage();
        assert!(small_alloc_usage.capacity_bytes > 0);

        // Calculate exact content bytes for small allocations
        let expected_small_content =
            std::mem::size_of::<u8>() + std::mem::size_of::<u16>() + std::mem::size_of::<u32>();
        assert_eq!(small_alloc_usage.content_bytes, expected_small_content);
        assert!(small_alloc_usage.total_bytes() == empty_usage.total_bytes());

        // Allocate a large array
        let _large_val = arena.allocate_val([42u64; 64]); // 512 bytes

        // Check memory usage after large allocation
        let large_alloc_usage = arena.memory_usage();
        assert!(large_alloc_usage.capacity_bytes > small_alloc_usage.capacity_bytes);

        // Calculate exact content bytes after large allocation
        let expected_large_content = expected_small_content + 64 * std::mem::size_of::<u64>();
        assert_eq!(large_alloc_usage.content_bytes, expected_large_content);
        assert!(large_alloc_usage.total_bytes() > small_alloc_usage.total_bytes());

        // Create a new arena with allocator to test reset
        let mut alloc = ArenaContainer::default();
        let arena1 = alloc.new_arena(0);

        // Make some allocations
        let _v1 = arena1.allocate_val(100u32);
        let _v2 = arena1.allocate_val(200u64);

        let usage1 = alloc.memory_usage();
        let expected_content1 = std::mem::size_of::<u32>() + std::mem::size_of::<u64>();
        assert_eq!(usage1.content_bytes, expected_content1);

        // Get a new arena (which resets the allocations)
        let arena2 = alloc.new_arena(1);

        // Allocate again
        let _v3 = arena2.allocate_val(300u32);

        let usage2 = alloc.memory_usage();
        let expected_content2 = std::mem::size_of::<u32>();
        assert_eq!(usage2.content_bytes, expected_content2);

        // Total memory should be the same after reset and reallocation
        // (since we're reusing memory)
        assert!(usage2.total_bytes() == usage1.total_bytes());
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
        let mut alloc = ArenaContainer::default();

        let arena = alloc.new_arena(0);

        // Grow u32 vecs (4-byte alignment)
        let vec_u32_a = arena.allocate_vec::<u32>();
        let vec_u32_b = arena.allocate_vec::<u32>();
        for i in 0..1000 {
            vec_u32_a.push(i);
        }
        for i in 0..500 {
            vec_u32_b.push(i);
        }

        let arena = alloc.new_arena(1);
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
        let mut alloc = ArenaContainer::default();
        let arena = alloc.new_arena(0);

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
        let mut alloc = ArenaContainer::default();
        let arena = alloc.new_arena(0);

        #[repr(align(32))]
        struct Test;

        let _ = arena.allocate_vec::<Test>();
    }

    #[test]
    fn test_memory_usage() {
        let mut alloc = ArenaContainer::default();
        let arena = alloc.new_arena(0);

        let initial_usage = arena.memory_usage();
        assert_eq!(initial_usage.content_bytes, 0);
        assert_eq!(initial_usage.capacity_bytes, 5 * 4 * 4);

        let vec1 = arena.allocate_vec::<u8>();
        vec1.extend_from_slice(&[1, 2, 3, 4, 5]);
        let _vec1_2 = arena.allocate_vec::<u8>();
        let _vec2 = arena.allocate_vec::<u32>();
        let _vec3 = arena.allocate_vec::<u64>();
        let _vec3_2 = arena.allocate_vec::<(u64, u64)>();

        let after_alloc = arena.memory_usage();
        assert!(after_alloc.capacity_bytes > 0);
        // Content bytes should be exactly 5 bytes (for vec1)
        assert_eq!(after_alloc.content_bytes, 5);

        let arena = alloc.new_arena(1);
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

        assert_eq!(after_data.content_bytes, expected_content_bytes);
        assert!(after_data.capacity_bytes >= expected_content_bytes);
        assert!(after_data.overhead_bytes > 0);
        assert!(after_data.memory_utilization_ratio() > 0.0);

        let arena = alloc.new_arena(2);
        let after_reset = arena.memory_usage();

        assert_eq!(
            after_reset.content_bytes, 0,
            "Content bytes should be 0 after reset"
        );
        assert!(
            after_reset.capacity_bytes > 0,
            "Capacity bytes should be preserved"
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
        let expected_content = 10 * 100 * std::mem::size_of::<u32>();
        assert_eq!(alloc_usage.content_bytes, expected_content);
        assert!(alloc_usage.capacity_bytes >= expected_content);
    }

    #[test]
    fn test_chunk_consolidation() {
        let mut alloc = ArenaContainer::default();

        // First arena: allocate vectors to force multiple chunks
        let arena = alloc.new_arena(0);

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
        let arena = alloc.new_arena(1);

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
        let arena = alloc.new_arena(2);

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
