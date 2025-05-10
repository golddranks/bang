use std::{cmp::max, mem::MaybeUninit};

use crate::{ErasedMax, ErasedMin, MemoryUsage};

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
        vec.resize_with(capacity, || MaybeUninit::uninit());
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

    fn cap(&self) -> usize {
        self.as_ptr().len()
    }

    fn cap_bytes(&self) -> usize {
        self.as_ptr().len() * size_of::<MaybeUninit<ErasedMax>>()
    }

    /// Returns a pointer to the value at the given index.
    ///
    /// # Safety
    ///
    /// 1. Caller must ensure that the pointer + the size of the pointed type
    ///    are in the bounds of the chunk
    /// 2. Caller must ensure that no other mutable references to the same slot exist
    unsafe fn get(&mut self, byte_offset: usize) -> *mut ErasedMin {
        let buf_start_ptr = (&raw mut *self.0) as *mut MaybeUninit<ErasedMin>;
        let obj_ptr = unsafe { buf_start_ptr.byte_add(byte_offset) };
        obj_ptr as *mut ErasedMin
    }

    fn new_from_chunks(chunks: &mut Vec<ValChunk>) -> ValChunk {
        let last_cap = chunks.last().map(|chunk| chunk.cap()).unwrap_or(1);
        // Why x4? x2 to accommodate the sizes of the earlier chunks, and another x2 for free space
        let mut new = Vec::with_capacity(last_cap * 4);
        for chunk in chunks.drain(..) {
            new.extend(chunk.0.into_iter());
        }
        new.resize_with(new.capacity(), || MaybeUninit::uninit());
        ValChunk(new.into_boxed_slice())
    }
}

pub(crate) struct ByAlign {
    last: usize,
    last_used_bytes: usize,
    chunks: Vec<ValChunk>,
    total_content_bytes: usize,
}

impl ByAlign {
    pub fn new() -> Self {
        ByAlign {
            last: 0,
            last_used_bytes: 0,
            chunks: vec![ValChunk::default()],
            total_content_bytes: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.last_used_bytes = 0;
        self.total_content_bytes = 0;
        if self.last > 0 {
            self.last = 0;
            let chunk = ValChunk::new_from_chunks(&mut self.chunks);
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

    pub fn allocate_val(&mut self, byte_size: usize) -> *mut ErasedMin {
        if self.capacity_over(byte_size) {
            self.grow(byte_size);
        }

        // Safety:
        // We increment last_used_bytes immediately after getting the reference
        // so future calls won't create aliasing mutable references.
        // When re-using values, lifetime restrictions on the allocator ensure
        // that earlier references are dead.
        let ptr = unsafe { self.chunks[self.last].get(self.last_used_bytes) };
        self.last_used_bytes += byte_size;
        ptr
    }

    pub(crate) fn memory_usage(&self, usage: &mut MemoryUsage) {
        usage.overhead_bytes += self.chunks.capacity() * size_of::<ValChunk>();

        let mut total_capacity = 0;
        for chunk in &self.chunks {
            total_capacity += chunk.cap_bytes();
        }
        usage.capacity_bytes += total_capacity;
        usage.content_bytes += self.total_content_bytes + self.last_used_bytes;
    }
}

#[cfg(test)]
mod test {
    use std::ptr::addr_eq;

    use super::*;

    #[test]
    fn test_val_basic() {
        let mut by_align = ByAlign::new();

        let val0_ptr = by_align.allocate_val(4) as *mut u32;
        let val0 = unsafe {
            val0_ptr.write(42);
            &mut *val0_ptr
        };
        let val1_ptr = by_align.allocate_val(4) as *mut u32;
        let val1 = unsafe {
            val1_ptr.write(43);
            &mut *val1_ptr
        };
        *val0 = 44;
        assert_eq!(*val0, 44);
        assert_eq!(*val1, 43);

        by_align.reset();

        let val3_ptr = by_align.allocate_val(4) as *mut u32;
        let val3 = unsafe {
            val3_ptr.write(45);
            &mut *val3_ptr
        };
        assert_eq!(*val3, 45);
        assert!(addr_eq(val0_ptr, val3_ptr));
    }
}
