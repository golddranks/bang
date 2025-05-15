use std::{
    fmt::Debug,
    marker::PhantomData,
    mem::{needs_drop, transmute},
    ops::{Deref, DerefMut},
    ptr::copy,
    slice,
};

use crate::{
    ErasedMax, MemoryUsage,
    val::{self, ByAlign},
    vec,
};

/// Short-lived arena implementation
///
/// Supports objects with aligments up to 16 bytes. This is the maximum
/// natural alignment for all supported platforms.
///
/// # Safety
///
/// This type alone is not safe to use by itself because of aliasing
/// restrictions of the mutable references borrowed out of this arena. This is
/// why it is meant to be safely wrapped by `Arena`, which borrows `ArenaGuard`
/// out with a restricted lifetime.
///
/// Specifically, any references returned by the allocation methods
/// (`alloc_val`, `alloc_vec`, `alloc_slice`, `alloc_iter`, `alloc_val_ignore_drop`,
/// `alloc_vec_ignore_drop`, `alloc_slice_ignore_drop`, `alloc_iter_ignore_drop`)
/// are not safe to use after the `reset` method is called. Reset splits the
/// use of this arena into monotonically increasing "allocation sequences",
/// indicated by the sequence number in `alloc_seq` field. Only the references
/// allocated on the current allocation sequence are safe to use.
///
/// Similarly, mutable vector references borrowed out of this arena are not
/// safe to use after the `memory_usage` method is called. However, mutable
/// slice references to the _contents_ of the vecs, and mutable references
/// to vals remain safe to use.
///
/// # Panics
///
/// The allocation methods (`alloc_val`, `alloc_vec`, `alloc_slice`,
/// `alloc_iter`, `alloc_val_ignore_drop`, `alloc_vec_ignore_drop`,
/// `alloc_slice_ignore_drop`, `alloc_iter_ignore_drop`) panic if attempting to
/// allocate a type with alignment that is not 1, 2, 4, 8, or 16. Attempting
/// to do so is API misuse and on caller's responsibility.
#[derive(Debug)]
pub struct ArenaGuard<'a> {
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

impl<'a> Drop for ArenaGuard<'a> {
    fn drop(&mut self) {
        // We need to manually drop the vecs to avoid memory leaks.
        // The vecs are not automatically dropped because they are stored as
        // type-erased vecs in `MaybeUninit<_>`. The destructors of the contents
        // of the vecs and single values are not called; this is part of the
        // library API contract, and considered safe in Rust. The default
        // allocating methods statically check that the allocated types are not
        // `Drop` to warn the user of any suprises.
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

impl<'a> ArenaGuard<'a> {
    pub(crate) fn new() -> Self {
        ArenaGuard {
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

    pub(crate) unsafe fn memory_usage(&self) -> MemoryUsage {
        let mut usage = MemoryUsage::default();
        usage.overhead_bytes += size_of::<ArenaGuard>();
        unsafe { self.vec_memory_usage(&mut usage) };
        self.val_memory_usage(&mut usage);
        usage
    }

    /// Returns the memory usage of vecs in this `Arena`.
    ///
    /// # Safety
    ///
    /// No aliasing: This method inspects the `capacity` and `len` fields of
    /// all vecs stored by this arena. The caller must ensure that no live
    /// mutable references (`&mut Vec<T>`) to the vecs stored by this arena are
    /// held. (Shared references (`&Vec<T>`) and mutable slice references to
    /// the contents of the vecs (`&mut [T]`) are OK.) Breaking this rule is
    /// Undefined Behavior.
    unsafe fn vec_memory_usage(&self, usage: &mut MemoryUsage) {
        let vec_aligns = [
            (1, &self.vec_align_1),
            (2, &self.vec_align_2),
            (4, &self.vec_align_4),
            (8, &self.vec_align_8),
            (16, &self.vec_align_16),
        ];

        for (align, by_align) in vec_aligns {
            // Safety:
            // No aliasing: This method can be called only when we have
            // exclusive access to the vectors.
            unsafe { by_align.memory_usage(align, self.alloc_seq, usage) };
        }
    }

    /// Returns the memory usage of vals in this `Arena`.
    pub(crate) fn val_memory_usage(&self, usage: &mut MemoryUsage) {
        let val_aligns = [
            &self.val_align_1,
            &self.val_align_2,
            &self.val_align_4,
            &self.val_align_8,
            &self.val_align_16,
        ];

        for by_align in val_aligns {
            by_align.memory_usage(usage);
        }
    }

    const fn get_vec_align(&mut self, align: usize) -> &mut vec::ByAlign {
        match align {
            1 => &mut self.vec_align_1,
            2 => &mut self.vec_align_2,
            4 => &mut self.vec_align_4,
            8 => &mut self.vec_align_8,
            16 => &mut self.vec_align_16,
            // Panics: API misuse; caller's responsibility
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
            // Panics: API misuse; caller's responsibility
            _ => panic!("Unsupported alignment"),
        }
    }

    pub fn alloc_vec<T>(&mut self) -> &'a mut Vec<T> {
        // Panics: Static assert, so no runtime panics are possible
        const { assert!(!needs_drop::<T>()) };
        self.alloc_vec_ignore_drop()
    }

    pub fn alloc_string(&mut self, str: &str) -> &'a mut String {
        // Panics: Static assert, so no runtime panics are possible
        const { assert!(size_of::<String>() == size_of::<Vec<u8>>()) }
        const { assert!(align_of::<String>() == align_of::<Vec<u8>>()) }

        let vec: &mut Vec<u8> = self.alloc_vec_ignore_drop();
        // Safety: String is defined as `pub struct String { vec: Vec<u8> }`
        // We ensure statically that the size and alignment of String and
        // Vec<u8> are the same. The allocated buffer also shares the
        // alignment and size of 1 byte, so the layouts are compatible.
        let s = unsafe { transmute::<&mut Vec<u8>, &mut String>(vec) };
        s.push_str(str);
        s
    }

    pub fn alloc_slice<T>(&mut self, slice: &[T]) -> &'a mut [T] {
        // Panics: Static assert, so no runtime panics are possible
        const { assert!(!needs_drop::<T>()) };
        self.alloc_slice_ignore_drop(slice)
    }

    pub fn alloc_str(&mut self, str: &str) -> &'a mut str {
        let slice = self.alloc_slice_ignore_drop(str.as_bytes());
        // Safety: `slice` originates from a `&str`, and is thus guaranteed
        // to be valid UTF-8.
        unsafe { str::from_utf8_unchecked_mut(slice) }
    }

    pub fn alloc_val<T>(&mut self, val: T) -> &'a mut T {
        // Panics: Static assert, so no runtime panics are possible
        const { assert!(!needs_drop::<T>()) };
        self.alloc_val_ignore_drop(val)
    }

    pub fn alloc_iter<T>(&mut self, iter: impl ExactSizeIterator<Item = T>) -> &'a mut [T] {
        // Panics: Static assert, so no runtime panics are possible
        const { assert!(!needs_drop::<T>()) };
        self.alloc_iter_ignore_drop(iter)
    }

    pub fn alloc_sink<'s, T>(&'s mut self) -> Sink<'s, 'a, T> {
        // Panics: Static assert, so no runtime panics are possible
        const { assert!(!needs_drop::<T>()) };
        self.alloc_sink_ignore_drop()
    }

    pub fn alloc_sink_ignore_drop<'s, T>(&'s mut self) -> Sink<'s, 'a, T> {
        let by_align = self.get_val_align(align_of::<T>());
        Sink {
            len: 0,
            storage: by_align,
            _marker: PhantomData,
        }
    }

    pub fn alloc_iter_ignore_drop<T>(
        &mut self,
        iter: impl ExactSizeIterator<Item = T>,
    ) -> &'a mut [T] {
        let item_byte_size = size_of::<T>();
        let by_align = self.get_val_align(align_of::<T>());
        // We can't trust this length, so measures for both it being too short
        // or too long are taken below. In case it's too short, we stop the
        // iteration to the allocated maximum with `take`. In case it's too
        // long, we call `shrink_val` to release excess memory.
        let len = iter.len();
        // Safety:
        // - Valid size: The used `by_align` is matched to the alignment of `T`.
        //   `T` size is always a multiple of that alignment per Rust's memory
        //   layout guarantees, and the allocated size is calculated to be a
        //   multiple of `T` size.
        // - No aliasing: References borrowed out from this arena during the
        //   past and future allocation sequences are ensured not to be live by
        //   the lifetime constraints of the arena API.
        let erased_ptr = unsafe { by_align.allocate_val(len * item_byte_size) };
        let start_typed_ptr = erased_ptr as *mut T;
        let mut typed_ptr = start_typed_ptr;
        let mut written = 0;
        for val in iter.take(len) {
            // Safety:
            // `erased_ptr`, converted to `typed_ptr` has size and alignment
            // that are ensured to be compatible with type `T` (size, by the
            // implementation of `allocate_val`, and alignment, by allocating
            // only values of the same alignment to the same alignment bucket).
            // The backing allocation is ensured to be unaliazed by
            // `allocate_val` and the lifetime constraints of the earlier and
            // future borrows. It is originally stored as unassuming
            // `MaybeUninit<_>`, and doesn't contain any live values.
            // The allocated length corresponds to the iterator length, and a
            // `take` iterator is created to ensure that the loop doesn't run
            // over the allocated length.
            unsafe {
                typed_ptr.write(val);
                typed_ptr = typed_ptr.add(1);
            }
            written += 1;
        }

        if written < len {
            let shrunk_by_bytes = (len - written) * item_byte_size;
            // Safety:
            // - No allocations between: There are no allocations between
            //   this `shrink_val` call and the `allocate_val` call above.
            // - Shrunk by valid size: `shrunk_by_bytes` is, by construction,
            //   constrained to be always 0 <= `shrunk_by_bytes` <= size of the
            //   last allocation, and always a multiple of the item size, and
            //   thus also a multiple of the alignment.
            // - No references: No references are created between the last
            //   allocation and the shrink operation.
            unsafe { by_align.shrink_val(erased_ptr, shrunk_by_bytes) };
        }

        // Safety: the slice is ensured to be of correct type `T` (valid memory
        // layout and initialized values), unaliased, and the length is the
        // same as the actual length produced by the iterator. The count of
        // produced values is checked, and only the initialized part of the
        // allocation is returned. (The uninitialized part is shrunk to fit by
        // `shrink_val`.)
        unsafe { slice::from_raw_parts_mut(start_typed_ptr, written) }
    }

    pub fn alloc_vec_ignore_drop<T>(&mut self) -> &'a mut Vec<T> {
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

    pub fn alloc_slice_ignore_drop<T>(&mut self, slice: &[T]) -> &'a mut [T] {
        let byte_size = size_of_val(slice);
        let by_align = self.get_val_align(align_of::<T>());

        // Safety:
        // - Valid size: The used `by_align` is matched to the alignment of `T`.
        //   `T` size is always a multiple of that alignment per Rust's memory
        //   layout guarantees, and the allocated size is calculated to be a
        //   multiple of `T` size.
        // - No aliasing: References borrowed out from this arena during the
        //   past and future allocation sequences are ensured not to be live by
        //   the lifetime constraints of the arena API.
        let erased_ptr = unsafe { by_align.allocate_val(byte_size) };
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

    pub fn alloc_val_ignore_drop<T>(&mut self, val: T) -> &'a mut T {
        let byte_size = size_of::<T>();
        let by_align = self.get_val_align(align_of::<T>());

        // Safety:
        // - Valid size: The used `by_align` is matched to the alignment of `T`.
        //   `T` size is always a multiple of that alignment per Rust's memory
        //   layout guarantees, and the allocated size is calculated to be a
        //   multiple of `T` size.
        // - No aliasing: References borrowed out from this arena during the
        //   past and future allocation sequences are ensured not to be live by
        //   the lifetime constraints of the arena API.
        let erased_ptr = unsafe { by_align.allocate_val(byte_size) };
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

    pub(crate) unsafe fn reset(&mut self, seq: usize) {
        self.alloc_seq = seq;
        self.val_align_1.reset();
        self.val_align_2.reset();
        self.val_align_4.reset();
        self.val_align_8.reset();
        self.val_align_16.reset();
    }
}

pub struct Sink<'s, 'a, T> {
    len: usize,
    storage: &'s mut ByAlign,
    _marker: PhantomData<&'a mut T>,
}

/// Sink is an allocator you can push values of same type into. The values
/// are stored continuously, and after allocating all the objects, sink can be
/// turned into a slice. Sink reserves `AreaGuard` until it is dropped or
/// turned into a slice.
impl<'s, 'a, T> Sink<'s, 'a, T> {
    pub fn push(&mut self, val: T) -> &mut T {
        let byte_size = size_of::<T>();
        let erased_ptr = unsafe { self.storage.allocate_continuous(self.len, byte_size) };
        let typed_ptr = erased_ptr as *mut T;
        self.len += 1;
        // Safety:
        // Compatible layout: erased_ptr has size and alignment that is ensured
        // to be compatible with T.
        // Exclusive access: `Sink` has, for its whole lifetime, exclusive
        // access to the memory of `ArenaGuard`, so the memory is not aliased.
        // Lifetime: The returned slice is valid for the self lifetime, so it is
        // invalidated as some other method of `Sink` is called; by the time
        // there are further allocations that might move the slice, the slice
        // returned by this method is already invalidated.
        unsafe {
            typed_ptr.write(val);
            &mut *typed_ptr
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let slice_end = self.storage.current_const() as *const T;
        let slice_start = slice_end.wrapping_sub(self.len);
        // Safety:
        // Exclusive access: `Sink` has, for its whole lifetime, exclusive
        // access to the memory of `ArenaGuard`, so the memory is not aliased.
        // Continuous allocation: `allocate_continuous` ensures that the memory
        // is allocated contiguously for the last `self.len` allocations made
        // by `Sink`, so it's safe to create a slice from the start to the end.
        // Lifetime: The returned slice is valid for the self lifetime, so it is
        // invalidated as some other method of `Sink` is called; by the time
        // there are further allocations that might move the slice, the slice
        // returned by this method is already invalidated.
        unsafe { slice::from_raw_parts(slice_start, self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let slice_end = self.storage.current_mut() as *mut T;
        let slice_start = slice_end.wrapping_sub(self.len);
        // Safety:
        // Exclusive access: `Sink` has, for its whole lifetime, exclusive
        // access to the memory of `ArenaGuard`, so the memory is not aliased.
        // Continuous allocation: `allocate_continuous` ensures that the memory
        // is allocated contiguously for the last `self.len` allocations made
        // by `Sink`, so it's safe to create a slice from the start to the end.
        // Lifetime: The returned slice is valid for the self lifetime, so it is
        // invalidated as some other method of `Sink` is called; by the time
        // there are further allocations that might move the slice, the slice
        // returned by this method is already invalidated.
        unsafe { slice::from_raw_parts_mut(slice_start, self.len) }
    }

    pub fn into_slice(self) -> &'a mut [T] {
        let slice_end = self.storage.current_mut() as *mut T;
        let slice_start = slice_end.wrapping_sub(self.len);
        // Safety:
        // Exclusive access: `Sink` has, for its whole lifetime, exclusive
        // access to the memory of `ArenaGuard`, so the memory is not aliased.
        // Continuous allocation: `allocate_continuous` ensures that the memory
        // is allocated contiguously for the last `self.len` allocations made
        // by `Sink`, so it's safe to create a slice from the start to the end.
        // Lifetime: The returned slice is valid for the arena lifetime 'a.
        // This means it can outlive `Sink`. This is fine, because `Sink` is
        // taken in by this method as an owned value, and will be dropped when
        // this method returns, so no further allocations can be made that might
        // move the slice.
        unsafe { slice::from_raw_parts_mut(slice_start, self.len) }
    }
}

impl<'s, 'a, T> Deref for Sink<'s, 'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'s, 'a, T> DerefMut for Sink<'s, 'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}
