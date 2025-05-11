use std::{
    marker::PhantomData,
    mem::{needs_drop, transmute},
    ptr::copy,
    slice,
};

#[cfg(any(test, doctest))]
mod tests;
mod val;
mod vec;

// Placeholder types for type erasure

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

    pub fn allocate_slice<T>(&mut self, slice: &[T]) -> &'a mut [T] {
        const { assert!(!needs_drop::<T>()) };
        self.allocate_slice_ignore_drop(slice)
    }

    pub fn allocate_val<T>(&mut self, val: T) -> &'a mut T {
        const { assert!(!needs_drop::<T>()) };
        self.allocate_val_ignore_drop(val)
    }

    pub fn allocate_iter<T>(&mut self, iter: impl ExactSizeIterator<Item = T>) -> &'a mut [T] {
        const { assert!(!needs_drop::<T>()) };
        self.allocate_iter_ignore_drop(iter)
    }

    pub fn allocate_iter_ignore_drop<T>(
        &mut self,
        iter: impl ExactSizeIterator<Item = T>,
    ) -> &'a mut [T] {
        let item_byte_size = size_of::<T>();
        let by_align = self.get_val_align(align_of::<T>());
        let len = iter.len();
        let erased_ptr = by_align.allocate_val(len * item_byte_size);
        let start_typed_ptr = erased_ptr as *mut T;
        let mut typed_ptr = start_typed_ptr;
        for val in iter.take(len) {
            unsafe {
                typed_ptr.write(val);
                typed_ptr = typed_ptr.add(1);
            }
        }

        unsafe { slice::from_raw_parts_mut(start_typed_ptr, len) }
    }

    pub fn allocate_vec_ignore_drop<T>(&mut self) -> &'a mut Vec<T> {
        let n_size = const { size_of::<T>() / align_of::<T>() };
        let seq = self.alloc_seq;
        let by_align = self.get_vec_align(align_of::<T>());
        let erased = by_align.allocate_vec(n_size, seq);
        let typed_ptr = erased as *mut Vec<ErasedMax> as *mut Vec<T>;

        // Safety: This cast between Vec types is safe because:
        // 1. Vec<T> and Vec<Erased> have identical memory layouts regardless of T
        // 2. We immediately set the Vec length to 0, so no contained values of
        //    the previous type will be incorrectly interpreted as type T
        let typed = unsafe { &mut *typed_ptr };

        // Safety: Setting len to 0 is safe even when reusing a vector that
        // previously held values of a different type because:
        // 1. Setting the length to 0, a minimal value, can never overflow the capacity
        // 2. We deliberately avoid running destructors on any potential existing elements
        //    (which would be unsound as they might be of a different, previous type)
        // 3. Not calling destructors is considered sound in Rust
        // Thus, any memory previously used by the vector becomes inaccessible
        // but remains allocated, allowing us to reuse the capacity.
        unsafe { typed.set_len(0) };
        typed
    }

    pub fn allocate_slice_ignore_drop<T>(&mut self, slice: &[T]) -> &'a mut [T] {
        let byte_size = size_of_val(slice);
        let by_align = self.get_val_align(align_of::<T>());
        let erased_ptr = by_align.allocate_val(byte_size);
        let typed_ptr = erased_ptr as *mut T;
        // Safety: erased_ptr has size and alignment that is ensured to be
        // compatible with T. The memory is originally stored as MaybeUninit<_>,
        // and it's safe to cast to a unique reference because it's not touched
        // via any other way as long as the borrow is live.
        unsafe {
            copy(slice.as_ptr(), typed_ptr, slice.len());
            slice::from_raw_parts_mut(typed_ptr, slice.len())
        }
    }

    pub fn allocate_val_ignore_drop<T>(&mut self, val: T) -> &'a mut T {
        let byte_size = size_of::<T>();
        let by_align = self.get_val_align(align_of::<T>());
        let erased_ptr = by_align.allocate_val(byte_size);
        let typed_ptr = erased_ptr as *mut T;
        // Safety: erased_ptr has size and alignment that is ensured to be
        // compatible with T. The memory is originally stored as MaybeUninit<_>,
        // and it's safe to cast to a unique reference because it's not touched
        // via any other way as long as the borrow is live.
        unsafe {
            typed_ptr.write(val);
            &mut *typed_ptr
        }
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
