use std::{marker::PhantomData, ptr::slice_from_raw_parts};

#[derive(Debug)]
#[repr(C)]
pub struct AllocManager {
    pool: Vec<Alloc<'static>>,
}

impl AllocManager {
    pub fn new() -> Self {
        AllocManager { pool: Vec::new() }
    }

    pub fn frame_alloc<'f>(&mut self) -> Alloc<'f> {
        if let Some(mut alloc) = self.pool.pop() {
            alloc.reset();
            alloc
        } else {
            Alloc::new()
        }
    }
}

/*
pub struct Buf<'a, T> {
    init: usize,
    buf: &'a mut [MaybeUninit<T>],
}

impl<'a, T> Buf<'a, T> {
    pub fn push(&mut self, value: T) {
        if self.init < self.buf.len() {
            self.buf[self.init].write(value);
            self.init += 1;
        }
    }

    pub fn to_slice(&self) -> &'a [T] {
        let slice = slice_from_raw_parts(self.buf.as_ptr() as *const T, self.init);
        unsafe { &*slice }
    }
}*/

pub struct FrameVec<'s, 'f, T> {
    vec: Vec<T>,
    _s_marker: PhantomData<&'s ()>,
    _f_marker: PhantomData<&'f ()>,
}

impl<'s, 'f, T> FrameVec<'s, 'f, T> {
    pub fn as_vec(&mut self) -> &mut Vec<T> {
        &mut self.vec
    }

    pub fn into_slice(self) -> &'f [T] {
        unsafe { &*slice_from_raw_parts(self.vec.as_ptr(), self.vec.len()) }
    }
}

#[derive(Debug)]
#[repr(C, align(8))]
struct Dummy([u8; 8]);

#[derive(Debug)]
#[repr(C)]
pub struct Alloc<'f> {
    in_use: usize,
    vecs: Vec<Vec<Dummy>>,
    _marker: PhantomData<&'f ()>,
    singles: Vec<Dummy>,
}

impl<'f> Alloc<'f> {
    pub fn frame_vec<'a, T>(&'a mut self) -> FrameVec<'a, 'f, T> {
        assert!(align_of::<T>() <= 8);
        if self.in_use == self.vecs.len() {
            let vec = Vec::new();
            self.vecs.push(vec);
        }
        let vec = &mut self.vecs[self.in_use];
        self.in_use += 1;

        let vec = unsafe { Vec::from_raw_parts(vec.as_mut_ptr() as *mut T, 0, vec.capacity()) };
        FrameVec {
            vec,
            _s_marker: PhantomData,
            _f_marker: PhantomData,
        }
    }

    pub fn frame<T>(&mut self, val: T) -> &'f mut T {
        assert!(align_of::<T>() <= 8);

        self.singles.push(Dummy);
        todo!() // TODO
    }

    pub fn new() -> Self {
        Alloc {
            in_use: 0,
            vecs: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn reset(&mut self) {
        self.vecs.clear();
    }
}
