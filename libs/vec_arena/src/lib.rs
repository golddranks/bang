use std::{
    alloc::{Layout, dealloc},
    marker::PhantomData,
    mem::{MaybeUninit, transmute},
};

// Erased can't be a zero-sized type, because Vec<Erased>'s capacity returns
// usize::MAX for a zero-sized type instead of an allocated capacity.
struct Erased {
    _padding: u8,
}

struct VecChunk(Box<[MaybeUninit<Vec<Erased>>]>);

impl Default for VecChunk {
    fn default() -> Self {
        Self::new(4)
    }
}

impl VecChunk {
    fn new(capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(capacity);
        vec.resize_with(capacity, || MaybeUninit::new(Vec::new()));
        VecChunk(vec.into_boxed_slice())
    }

    fn as_ptr(&self) -> *const [MaybeUninit<Vec<Erased>>] {
        // A workaround around current (as of Rust 1.86) Miri limitations:
        // converting the box to a pointer retags the contents of the slice
        // and causes an UB warning. Incidentally `&raw mut *boxed_slice` works
        // here even if `&raw const *boxed_slice` would be semantically more correct,
        // so we use a detour route & -> *const -> *mut -> deref -> *mut -> *const
        // as a workaround.
        //
        // See https://github.com/rust-lang/miri/issues/4317
        let chunk_ptr = &raw const *self as *mut Self;
        unsafe { &raw mut *(*chunk_ptr).0 }
    }

    fn cap(&self) -> usize {
        self.as_ptr().len()
    }

    /// Returns a mutable reference to the vector at the given index.
    ///
    /// # Safety
    ///
    /// Caller must ensure that no other mutable references to the same slot exist
    unsafe fn get(&mut self, idx: usize) -> &mut Vec<Erased> {
        assert!(idx < self.cap());
        let first_slot_ptr = (&raw mut *self.0) as *mut MaybeUninit<Vec<Erased>>;
        // Safety: idx must be less than the capacity (asserted above)
        let slot = unsafe { &mut *first_slot_ptr.add(idx) };
        // Safety: slot must be initialized (happens on chunk allocation)
        unsafe { slot.assume_init_mut() }
    }

    fn new_from_chunks(chunks: &mut Vec<VecChunk>) -> VecChunk {
        let last_cap = chunks.last().map(|chunk| chunk.cap()).unwrap_or(1);
        // Why x4? x2 to accommodate the sizes of the earlier chunks, and another x2 for free space
        let mut new = Vec::with_capacity(last_cap * 4);
        for chunk in chunks.drain(..) {
            new.extend(chunk.0.into_iter());
        }
        new.resize_with(new.capacity(), || MaybeUninit::new(Vec::new()));
        VecChunk(new.into_boxed_slice())
    }
}

struct BySize {
    seq: usize,
    last: usize,
    last_used_count: usize,
    vec_chunks: Vec<VecChunk>,
}

impl BySize {
    const fn new() -> Self {
        BySize {
            seq: 0,
            last: 0,
            last_used_count: 0,
            vec_chunks: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.last_used_count = 0;
        if self.last > 0 {
            self.last = 0;
            let chunk = VecChunk::new_from_chunks(&mut self.vec_chunks);
            self.vec_chunks.push(chunk);
        }
    }
    fn capacity_full(&self) -> bool {
        self.vec_chunks[self.last].cap() == self.last_used_count
    }

    fn grow(&mut self) {
        let last_cap = self.vec_chunks[self.last].cap();
        self.vec_chunks.push(VecChunk::new(last_cap * 2));
        self.last += 1;
        self.last_used_count = 0;
    }

    fn get_new(&mut self, seq: usize) -> &mut Vec<Erased> {
        if self.vec_chunks.is_empty() {
            self.vec_chunks.push(VecChunk::default());
        }

        if seq > self.seq {
            self.seq = seq;
            self.reset();
        }

        if self.capacity_full() {
            self.grow();
        }

        let chunk = &mut self.vec_chunks[self.last];
        // Safety:
        // We increment last_used_count immediately after getting the reference
        // so future calls won't create aliasing mutable references.
        // When re-using the Vec, lifetime restrictions on the allocator ensure
        // that earlier references are dead.
        let slot = unsafe { chunk.get(self.last_used_count) };
        self.last_used_count += 1;
        slot
    }
}

struct ByAlign {
    sizes: Vec<BySize>,
}

impl ByAlign {
    const fn new() -> Self {
        ByAlign { sizes: Vec::new() }
    }

    fn allocate_vec(&mut self, n_size: usize, seq: usize) -> &mut Vec<Erased> {
        if n_size >= self.sizes.len() {
            self.sizes.resize_with(n_size + 1, BySize::new);
        }
        self.sizes[n_size].get_new(seq)
    }
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

pub struct VecArena<'a> {
    alloc_seq: usize,
    align_1: ByAlign,
    align_2: ByAlign,
    align_4: ByAlign,
    align_8: ByAlign,
    align_16: ByAlign,
    _lifetime: PhantomData<&'a mut Erased>,
}

impl<'a> Drop for VecArena<'a> {
    fn drop(&mut self) {
        let aligns = [
            (1, &mut self.align_1),
            (2, &mut self.align_2),
            (4, &mut self.align_4),
            (8, &mut self.align_8),
            (16, &mut self.align_16),
        ];
        for (align, by_align) in aligns {
            for (n, by_size) in by_align.sizes.drain(..).enumerate() {
                let element_size = n * align;
                for chunk in by_size.vec_chunks {
                    for mut slot in chunk.0 {
                        let erased = unsafe { slot.assume_init_mut() };
                        if erased.capacity() > 0 {
                            let alloc_size = element_size * erased.capacity();
                            let layout =
                                Layout::from_size_align(alloc_size, align).expect("UNREACHABLE");
                            let ptr = erased.as_mut_ptr() as *mut u8;
                            unsafe { dealloc(ptr, layout) };
                        }
                    }
                }
            }
        }
    }
}

impl<'a> VecArena<'a> {
    const fn new() -> Self {
        VecArena {
            alloc_seq: 1,
            align_1: ByAlign::new(),
            align_2: ByAlign::new(),
            align_4: ByAlign::new(),
            align_8: ByAlign::new(),
            align_16: ByAlign::new(),
            _lifetime: PhantomData,
        }
    }

    fn memory_usage(&'a self) -> MemoryUsage {
        let mut usage = MemoryUsage::default();

        let aligns = [
            (1, &self.align_1),
            (2, &self.align_2),
            (4, &self.align_4),
            (8, &self.align_8),
            (16, &self.align_16),
        ];

        usage.overhead_bytes += size_of::<VecArena>();

        for (alignment, by_align) in aligns {
            usage.overhead_bytes += by_align.sizes.capacity() * size_of::<BySize>();
            for (n_size, by_size) in by_align.sizes.iter().enumerate() {
                let element_size = n_size * alignment;
                usage.chunk_count += by_size.vec_chunks.len();
                usage.overhead_bytes += by_size.vec_chunks.capacity() * size_of::<VecChunk>();
                let mut by_size_slots = 0;
                for chunk in by_size.vec_chunks.iter() {
                    by_size_slots += chunk.cap();
                    usage.overhead_bytes += size_of_val(&*chunk.0);

                    for slot_idx in 0..chunk.cap() {
                        let vec = unsafe { &*chunk.0[slot_idx].assume_init_ref() };
                        usage.vector_capacity_bytes += vec.capacity() * element_size;
                        if self.alloc_seq == by_size.seq {
                            usage.vector_content_bytes += vec.len() * element_size;
                        }
                    }
                }

                let last_cap = by_size
                    .vec_chunks
                    .last()
                    .map(|chunk| chunk.cap())
                    .unwrap_or(0);

                usage.total_slots += by_size_slots;
                if self.alloc_seq == by_size.seq {
                    usage.used_slots += by_size_slots - last_cap + by_size.last_used_count;
                }
            }
        }

        usage
    }

    const fn get_align(&mut self, align: usize) -> &mut ByAlign {
        match align {
            1 => &mut self.align_1,
            2 => &mut self.align_2,
            4 => &mut self.align_4,
            8 => &mut self.align_8,
            16 => &mut self.align_16,
            _ => panic!("Unsupported alignment"),
        }
    }

    pub fn allocate_vec<T>(&mut self) -> &'a mut Vec<T> {
        let n_size = const { size_of::<T>() / align_of::<T>() };
        let seq = self.alloc_seq;
        let by_align = self.get_align(align_of::<T>());
        let erased = by_align.allocate_vec(n_size, seq);
        let typed_ptr = erased as *mut Vec<Erased> as *mut Vec<T>;

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
    arena: VecArena<'static>,
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
            arena: VecArena::new(),
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

    pub fn new_arena<'a>(&'a mut self) -> &'a mut VecArena<'a> {
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
        let mut arena = VecArena::new();
        let _vec = arena.allocate_vec::<u32>();

        drop(arena);
    }

    #[test]
    fn test_empty_allocation() {
        let arena = VecArena::new();

        drop(arena);
    }

    #[test]
    fn test_basic_allocation() {
        let mut arena = VecArena::new();

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
        let mut arena = VecArena::new();

        let vec1 = arena.allocate_vec::<u8>();
        let vec2 = arena.allocate_vec::<u8>();

        vec1.extend_from_slice(&[1, 2, 3]);
        vec2.extend_from_slice(&[4, 5, 6]);

        assert_eq!(vec1, &[1, 2, 3]);
        assert_eq!(vec2, &[4, 5, 6]);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut arena = VecArena::new();

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
        let mut arena = VecArena::new();

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
        let mut arena = VecArena::new();

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
        let mut arena = VecArena::new();

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
