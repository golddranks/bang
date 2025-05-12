use std::{cmp::max, mem::MaybeUninit};

use crate::{ErasedMax, ErasedMin, MemoryUsage};

#[derive(Debug)]
struct ValChunk(Box<[MaybeUninit<ErasedMax>]>);

impl Default for ValChunk {
    fn default() -> Self {
        Self::new(size_of::<MaybeUninit<ErasedMax>>())
    }
}

impl ValChunk {
    fn new(cap_bytes: usize) -> Self {
        let capacity = cap_bytes.div_ceil(size_of::<MaybeUninit<ErasedMax>>());
        let mut vec = Vec::with_capacity(capacity);
        vec.resize_with(capacity, MaybeUninit::uninit);
        ValChunk(vec.into_boxed_slice())
    }

    fn as_ptr(&self) -> *const [MaybeUninit<ErasedMax>] {
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

    fn cap_bytes(&self) -> usize {
        self.as_ptr().len() * size_of::<MaybeUninit<ErasedMax>>()
    }

    /// Returns a mutable reference to the vector at the given index.
    ///
    /// # Safety
    ///
    /// - No aliasing: Caller must ensure that no other references to the
    ///   pointed memory exist, and no other such references are created until
    ///   the pointer and possible references derived from it are no longer used.
    /// - In bounds: Caller must ensure that the pointer + the size of the
    ///   pointed type are in the bounds of the chunk, even after the pointer
    ///   is cast from the temporary ErasedMin to the final type
    unsafe fn get(&mut self, byte_offset: usize) -> *mut ErasedMin {
        let buf_start_ptr = (&raw mut *self.0) as *mut MaybeUninit<ErasedMin>;
        // Safety: caller's responsibility as documented
        let obj_ptr = unsafe { buf_start_ptr.byte_add(byte_offset) };
        obj_ptr as *mut ErasedMin
    }

    fn new_from_chunks(chunks: &mut Vec<ValChunk>, prev_content_bytes: usize) -> ValChunk {
        let mut new = Vec::with_capacity(prev_content_bytes * 2);
        for chunk in chunks.drain(..) {
            new.extend(chunk.0.into_iter());
        }
        new.resize_with(new.capacity(), MaybeUninit::uninit);
        ValChunk(new.into_boxed_slice())
    }
}

#[derive(Debug)]
pub(crate) struct ByAlign {
    last: usize,
    last_used_bytes: usize,
    chunks: Vec<ValChunk>,
    total_content_bytes: usize,
}

impl ByAlign {
    pub(crate) fn new() -> Self {
        ByAlign {
            last: 0,
            last_used_bytes: 0,
            chunks: vec![ValChunk::default()],
            total_content_bytes: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.last_used_bytes = 0;
        let prev_content_bytes = self.total_content_bytes;
        self.total_content_bytes = 0;
        if self.last > 0 {
            self.last = 0;
            let chunk = ValChunk::new_from_chunks(&mut self.chunks, prev_content_bytes);
            self.chunks.push(chunk);
        }
    }

    fn capacity_over(&self, byte_size: usize) -> bool {
        self.last_used_bytes + byte_size > self.chunks[self.last].cap_bytes()
    }

    fn grow(&mut self, byte_size: usize) {
        self.total_content_bytes += self.last_used_bytes;

        let last_cap_bytes = self.chunks[self.last].cap_bytes();
        // Ensure new capacity is at least double the size we need
        let new_cap_bytes = max(last_cap_bytes * 2, byte_size * 2);
        self.chunks.push(ValChunk::new(new_cap_bytes));
        self.last += 1;
        self.last_used_bytes = 0;
    }

    /// Allocate a value of the given size in bytes.
    ///
    /// # Safety
    ///
    /// - Valid size: You must ensure that the argument `byte_size` is valid.
    ///   In particular, you must ensure that all the values stored in the
    ///   same `ByAlign` object have the same alignment, and `byte_size` is a
    ///   multiple of that alignment.
    /// - No aliasing: If you create references from returned `*mut ErasedMin`,
    ///   you must ensure that they are no longer live after calling `reset`.
    pub(crate) unsafe fn allocate_val(&mut self, byte_size: usize) -> *mut ErasedMin {
        if self.capacity_over(byte_size) {
            self.grow(byte_size);
        }

        // Safety:
        // - No aliasing: We increment last_used_bytes immediately after getting
        //   the reference so future calls won't create aliasing references.
        //   When re-using values, lifetime restrictions on the Area ensure
        //   that earlier references are dead.
        // - In bounds: The pointer offset (`last_used_bytes`) + the size of
        //   the pointed value (`byte_size`) are checked to always be in bounds
        //   by the `capacity_over()` method.
        let ptr = unsafe { self.chunks[self.last].get(self.last_used_bytes) };
        self.last_used_bytes += byte_size;
        ptr
    }

    /// Shrinks last allocation
    ///
    /// The `last_alloc_ptr` argument is used only in debug assertions.
    ///
    /// # Safety
    ///
    /// - No allocations between: Caller must call `shrink_val` as soon as
    ///   possible after the last allocation. In particular, there must be no
    ///   other allocations between the last allocation and the call to
    ///   `shrink_val`.
    /// - Shrunk by valid size: `by` must be less than or equal to the size
    ///   of the last allocation, and a multiple of the `ByAlign` alignment.
    /// - No references: There must be no live references to the shrinked
    ///   part.
    pub(crate) unsafe fn shrink_val(&mut self, last_alloc_ptr: *mut ErasedMin, by: usize) {
        // Check that the pointer is within the bounds of the last chunk,
        // and `by` is less than the allocation size
        #[cfg(debug_assertions)]
        {
            let start_addr = self.chunks[self.last].0.as_ptr().addr();
            let end_addr = start_addr + self.last_used_bytes;
            debug_assert!(start_addr <= last_alloc_ptr.addr());
            debug_assert!(last_alloc_ptr.addr() <= end_addr - by);
        }
        self.last_used_bytes -= by;
    }

    fn cap_bytes(&self) -> usize {
        self.chunks.iter().map(|c| c.cap_bytes()).sum()
    }

    fn content_bytes(&self) -> usize {
        self.total_content_bytes + self.last_used_bytes
    }

    pub(crate) fn memory_usage(&self, usage: &mut MemoryUsage) {
        usage.overhead_bytes += self.chunks.capacity() * size_of::<ValChunk>();
        usage.capacity_bytes += self.cap_bytes();
        usage.content_bytes += self.content_bytes();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ptr::addr_eq;

    #[test]
    fn test_val_basic() {
        let mut by_align = ByAlign::new();

        let helper = |by_align: &mut ByAlign, val: u32| {
            let val_ptr = unsafe { by_align.allocate_val(size_of::<u32>()) } as *mut u32;
            let val = unsafe {
                val_ptr.write(val);
                &mut *val_ptr
            };
            (val, val_ptr)
        };

        let (val0, val0_ptr) = helper(&mut by_align, 42);
        let (val1, _) = helper(&mut by_align, 43);

        *val0 = 45;
        assert_eq!(*val0, 45);
        assert_eq!(*val1, 43);

        by_align.reset();

        let (val2, val2_ptr) = helper(&mut by_align, 44);
        assert_eq!(*val2, 44);
        assert!(addr_eq(val0_ptr, val2_ptr));
    }

    fn alloc(by_align: &mut ByAlign, size: usize) {
        unsafe { by_align.allocate_val(size) };
    }

    #[test]
    fn test_val_growth_patterns() {
        let mut align = ByAlign::new();
        let align = &mut align;
        assert_eq!(align.cap_bytes(), 16); // Preallocated
        assert_eq!(align.content_bytes(), 0); // No content yet
        for _ in 0..16 {
            alloc(align, 1); // Allocating 16 x 1 (u8)
        }
        assert_eq!(align.cap_bytes(), 16); // Full
        assert_eq!(align.content_bytes(), 16); // 16 bytes of content
        alloc(align, 1); // Overflowing by 1
        assert_eq!(align.cap_bytes(), 48); // Grew by 32 (16 + 32 = 48)
        assert_eq!(align.content_bytes(), 17); // 16 + 1 = 17 bytes of content
        alloc(align, 31); // Allocating [u8; 31], Filled to the brim
        assert_eq!(align.cap_bytes(), 48); // Full again
        assert_eq!(align.content_bytes(), 48); // 17 + 31 = 48 bytes of content
        alloc(align, 1); // Overflowing by 1, Only 1/64 filled
        assert_eq!(align.cap_bytes(), 112); // Grew by 64 (48 + 64 = 112)
        assert_eq!(align.content_bytes(), 49); // 48 + 1 = 49 bytes of content
        alloc(align, 64); // Overflowing by 1, 64/128 filled
        assert_eq!(align.cap_bytes(), 240); // Grew by 128 (112 + 128 = 240)
        assert_eq!(align.content_bytes(), 113); // 49 + 64 = 113 bytes of content
        alloc(align, 130); // Overflowing by under next increment (256), but...
        // over the current capacity: 2*130 = 260, rounded up to 272 (nearest multiple of 16)
        assert_eq!(align.cap_bytes(), 512); // Grew by 272 (240 + round(2*130) = 512)
        assert_eq!(align.content_bytes(), 243); // 113 + 130 = 243 bytes of content
        alloc(align, 550); // Overflowing by over next increment (544)
        assert_eq!(align.cap_bytes(), 1616); // Grew by 1104 (rounded up) (512 + round(2*550) = 1616)
        assert_eq!(align.content_bytes(), 793); // 243 + 550 = 793 bytes of content
        alloc(align, 554); // Filled to the brim
        assert_eq!(align.cap_bytes(), 1616); // Full again
        assert_eq!(align.content_bytes(), 1347); // 793 + 554 = 1347 bytes of content
        alloc(align, 1);
        assert_eq!(align.cap_bytes(), 3824); // Grew by 2208 (1616 + 2*1104 = 3824)
        assert_eq!(align.content_bytes(), 1348); // 1347 + 1 = 1348 bytes of content
    }

    #[test]
    fn test_val_reset() {
        let mut align = ByAlign::new();
        let align = &mut align;
        assert_eq!(align.cap_bytes(), 16); // Preallocated
        assert_eq!(align.content_bytes(), 0); // No content yet
        alloc(align, 100); // [u16; 50]
        assert_eq!(align.cap_bytes(), 224); // Grew by 208 (16 + round(2*100) = 216)
        assert_eq!(align.content_bytes(), 100); // 100 bytes of content

        align.reset();

        assert_eq!(align.cap_bytes(), 224); // The other than the bigger slice are dropped
        assert_eq!(align.content_bytes(), 0); // Content reset to 0
        alloc(align, 220); // Would overflow a 16-byte chunk
        assert_eq!(align.cap_bytes(), 224); // But it fits this!
        assert_eq!(align.content_bytes(), 220); // 220 bytes of content
    }
}
