use std::{
    alloc::{Layout, dealloc},
    mem::MaybeUninit,
};

use crate::{ErasedMax as Erased, MemoryUsage};

#[derive(Debug)]
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
    /// - No aliasing: Caller must ensure that no other references to the
    ///   pointed slot exist, and no other such references are created until
    ///   the pointer and possible references derived from it are no longer used.
    /// - In bounds: Caller must also ensure that the index is in bounds.
    unsafe fn get(&mut self, idx: usize) -> &mut Vec<Erased> {
        // Panics: this function is called only internally, and the invariant
        // of index being less than capacity is always upheld. In case it's not,
        // that's a bug.
        debug_assert!(idx < self.cap());
        let first_slot_ptr = self.as_ptr() as *mut MaybeUninit<Vec<Erased>>;
        // Safety: idx must be less than the capacity (debug-asserted above but
        // ultimately caller's responsibility, as documented), and
        // there must be no other aliasing references to the slot. (Caller's
        // responsibility, as documented)
        let slot = unsafe { &mut *first_slot_ptr.add(idx) };
        // Safety: slot is always initialized (happens on chunk allocation)
        unsafe { slot.assume_init_mut() }
    }

    fn new_from_chunks(chunks: &mut Vec<VecChunk>) -> VecChunk {
        let last_cap = chunks.last().map(|chunk| chunk.cap()).unwrap_or(1);
        // Why x4? x2 to accommodate the sizes of the earlier chunks,
        // and another x2 for free space
        let mut new = Vec::with_capacity(last_cap * 4);
        for chunk in chunks.drain(..) {
            new.extend(chunk.0.into_iter());
        }
        new.resize_with(new.capacity(), || MaybeUninit::new(Vec::new()));
        VecChunk(new.into_boxed_slice())
    }
}

#[derive(Debug)]
struct BySize {
    seq: usize,
    last: usize,
    last_used_count: usize,
    chunks: Vec<VecChunk>,
}

impl BySize {
    const fn new() -> Self {
        BySize {
            seq: 0,
            last: 0,
            last_used_count: 0,
            chunks: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.last_used_count = 0;
        if self.last > 0 {
            self.last = 0;
            let chunk = VecChunk::new_from_chunks(&mut self.chunks);
            self.chunks.push(chunk);
        }
    }
    fn capacity_full(&self) -> bool {
        self.chunks[self.last].cap() == self.last_used_count
    }

    fn grow(&mut self) {
        let last_cap = self.chunks[self.last].cap();
        self.chunks.push(VecChunk::new(last_cap * 2));
        self.last += 1;
        self.last_used_count = 0;
    }

    fn get_new(&mut self, seq: usize) -> &mut Vec<Erased> {
        if self.chunks.is_empty() {
            self.chunks.push(VecChunk::default());
        }

        if seq > self.seq {
            self.seq = seq;
            self.reset();
        }

        if self.capacity_full() {
            self.grow();
        }

        let chunk = &mut self.chunks[self.last];
        // Safety:
        // - No aliasing: we increment last_used_count immediately after getting
        //   the reference so future calls won't create aliasing references.
        //   When re-using the Vec, lifetime restrictions on the Area ensure
        //   that earlier references are dead.
        // - In bounds: `last_used_count` is checked to always be in bounds by
        //   the `capacity_full()` method.
        let slot = unsafe { chunk.get(self.last_used_count) };
        self.last_used_count += 1;
        slot
    }
}

#[derive(Debug)]
pub(crate) struct ByAlign {
    sizes: Vec<BySize>,
}

impl ByAlign {
    pub const fn new() -> Self {
        ByAlign { sizes: Vec::new() }
    }

    pub fn allocate_vec(&mut self, n_size: usize, seq: usize) -> &mut Vec<Erased> {
        if n_size >= self.sizes.len() {
            self.sizes.resize_with(n_size + 1, BySize::new);
        }
        self.sizes[n_size].get_new(seq)
    }

    pub(crate) fn drop(&mut self, align: usize) {
        for (n, by_size) in self.sizes.drain(..).enumerate() {
            let element_size = n * align;
            for chunk in by_size.chunks {
                for mut slot in chunk.0 {
                    // Safety: slot is always initialized (happens on chunk allocation)
                    let erased = unsafe { slot.assume_init_mut() };
                    if erased.capacity() > 0 {
                        let alloc_size = element_size * erased.capacity();
                        // UNREACHABLE: The align sizes 1, 2, 4, 8, 16 are always valid
                        // and alloc_size is calculated from the actually allocated size
                        // so the layout is always valid
                        let layout = Layout::from_size_align(alloc_size, align)
                            .expect("UNREACHABLE: always valid");
                        let ptr = erased.as_mut_ptr() as *mut u8;
                        // Safety: ptr is originally allocated by `Vec`, held
                        // by this arena. All such vecs use the global
                        // allocator, which is also what `dealloc` uses.
                        // Layout is calculated to match the align and size of
                        // the allocation held by the vec.
                        unsafe { dealloc(ptr, layout) };
                    }
                }
            }
        }
    }

    /// Returns the memory usage of vecs in this `ByAlign`.
    ///
    /// # Safety
    ///
    /// No aliasing: This method inspects the `capacity` and `len` fields of
    /// all vecs stored by this arena. The caller must ensure that no live
    /// mutable references (`&mut Vec<T>`) to the vecs stored by this arena are
    /// held. (Shared references (`&Vec<T>`) and mutable slice references to
    /// the contents of the vecs (`&mut [T]`) are OK.) Breaking this rule is
    /// Undefined Behavior.
    pub(crate) unsafe fn memory_usage(&self, align: usize, seq: usize, usage: &mut MemoryUsage) {
        usage.overhead_bytes += self.sizes.capacity() * size_of::<BySize>();
        for (n_size, by_size) in self.sizes.iter().enumerate() {
            let element_size = n_size * align;
            usage.overhead_bytes += by_size.chunks.capacity() * size_of::<VecChunk>();

            for chunk in &by_size.chunks {
                let len = chunk.as_ptr().len();
                usage.overhead_bytes += len * size_of::<MaybeUninit<Vec<Erased>>>();

                let start = chunk.as_ptr() as *const MaybeUninit<Vec<Erased>>;
                for offset in 0..len {
                    // Safety: Slot is in bounds for all offsets 0..len.
                    let slot = unsafe { start.add(offset) };
                    // Safety (dereference):
                    // 1. Slot is ensured to be in bounds.
                    // 2. There are no live mutable references to the slot
                    //    (caller's responsibility).
                    // Safety (assume_init):
                    // Slot is always initialized (happens on chunk allocation).
                    let vec = unsafe { (*slot).assume_init_ref() };
                    usage.capacity_bytes += vec.capacity() * element_size;
                    if seq == by_size.seq {
                        usage.content_bytes += vec.len() * element_size;
                    }
                }
            }
        }
    }
}
