use std::{
    any::type_name,
    fmt::Debug,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::slice_from_raw_parts,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FrameVec<'slf, 'frame, T> {
    vec: &'slf mut Vec<T>,
    _frame_lifetime: PhantomData<&'frame ()>,
}

impl<'s, 'f, T> Deref for FrameVec<'s, 'f, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        self.vec
    }
}

impl<'s, 'f, T> DerefMut for FrameVec<'s, 'f, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.vec
    }
}

impl<'s, 'f, T> FrameVec<'s, 'f, T> {
    pub fn into_slice(self) -> &'f [T] {
        // SAFETY: Alloc guarantees that the allocated buffer (Vec<P>) will
        // not be reallocated or dropped for 'f, that is, until the whole
        // Alloc is reset and returned to the AllocManager.
        // This is ensured by the fact that the Vec<P> length/capacity is not
        // modified after FrameVec is turned into a slice.
        unsafe { &*slice_from_raw_parts(self.vec.as_ptr(), self.vec.len()) }
    }
}

#[derive(Debug)]
#[repr(C)]
struct Vecs<P> {
    vecs: Vec<(usize, Vec<MaybeUninit<P>>)>,
    in_use: usize,
}

#[derive(Debug)]
#[repr(C)]
struct Singles<P> {
    slices: Vec<Box<[MaybeUninit<P>]>>,
    in_use: usize,
    latest_filled_up_to: usize,
}

#[repr(C)]
pub struct Alloc<'frame> {
    pub alloc_seq: usize,
    _frame_lifetime: PhantomData<&'frame ()>,
    vecs16: Vecs<u128>,
    vecs8: Vecs<u64>,
    vecs4: Vecs<u32>,
    vecs2: Vecs<u16>,
    vecs1: Vecs<u8>,
    singles16: Singles<u128>,
    singles8: Singles<u64>,
    singles4: Singles<u32>,
    singles2: Singles<u16>,
    singles1: Singles<u8>,
}

impl<P> Drop for Vecs<P> {
    fn drop(&mut self) {
        for (type_size, vec) in &mut self.vecs {
            Vecs::reinterpret::<MaybeUninit<P>>(type_size, vec);
        }
    }
}

impl<P> Vecs<P> {
    fn size(&self) -> usize {
        self.vecs
            .iter()
            .map(|(type_size, vec)| vec.capacity() * *type_size)
            .sum()
    }

    fn reinterpret<'s, T>(
        type_size: &'s mut usize,
        vec: &'s mut Vec<MaybeUninit<P>>,
    ) -> &'s mut Vec<T> {
        assert!(size_of::<T>() != 0);
        vec.clear();

        let byte_size = vec.len() * *type_size;
        let byte_cap = vec.capacity() * *type_size;
        let new_len = byte_size / size_of::<T>(); // The divisor is known compile-time and is a power of 2
        let new_cap = byte_cap / size_of::<T>(); // Likewise
        let new_ptr = vec.as_mut_ptr() as *mut T;

        *type_size = size_of::<T>();
        let vec = &raw mut *vec as *mut Vec<T>;

        unsafe {
            vec.write(Vec::from_raw_parts(new_ptr, new_len, new_cap));
            &mut *vec
        }
    }

    fn get_new<T>(&mut self) -> &mut Vec<T> {
        assert_eq!(align_of::<T>(), align_of::<P>());
        if self.in_use == self.vecs.len() {
            #[cfg(feature = "eprint_alloc")]
            eprintln!("Adding a new Vec in a Vec<{}> allocator.", type_name::<P>());
            let vec = Vec::new();
            self.vecs.push((0, vec));
        }
        let (type_size, vec) = &mut self.vecs[self.in_use];
        self.in_use += 1;
        Vecs::reinterpret(type_size, vec)
    }

    fn reset(&mut self) {
        self.in_use = 0;
    }

    fn new() -> Self {
        Self {
            vecs: Vec::new(),
            in_use: 0,
        }
    }
}

impl<P> Singles<P> {
    fn size(&self) -> usize {
        self.slices
            .iter()
            .map(|slice| slice.len() * size_of::<P>())
            .sum()
    }

    fn make_slice(size: usize) -> Box<[MaybeUninit<P>]> {
        #[cfg(feature = "eprint_alloc")]
        eprintln!(
            "Allocating a slice of size {size} for a Singles<{}> allocator.",
            type_name::<P>()
        );
        let mut new_slice: Vec<MaybeUninit<P>> = Vec::with_capacity(size);
        // SAFETY: exposing uninitialized memory is OK, because the contents is MaybeUninit<_>
        unsafe { new_slice.set_len(size) };
        new_slice.into_boxed_slice()
    }

    fn get_slice_idx(&self, min_size: usize) -> Option<usize> {
        let mut idx = self.in_use;
        while idx < self.slices.len() {
            let slice = &self.slices[idx];
            let slice_big_enough = slice.len() - self.latest_filled_up_to >= min_size;
            if slice_big_enough {
                return Some(idx);
            } else {
                idx += 1;
            }
        }
        None
    }

    fn get_new<T>(&mut self) -> *mut T {
        assert_eq!(align_of::<T>(), align_of::<P>());
        let t_size_in_p_units = size_of::<T>().div_ceil(size_of::<P>()); // Division should happen compile time
        match self.get_slice_idx(t_size_in_p_units) {
            Some(slice_idx) => {
                self.in_use = slice_idx;
            }
            None => {
                self.in_use = self.slices.len();
                let biggest_size = self.slices.last().expect("UNREACHABLE").len();
                let new_size = biggest_size.max(t_size_in_p_units) * 2;
                let new_slice = Self::make_slice(new_size);
                self.slices.push(new_slice);
                self.latest_filled_up_to = 0;
            }
        }

        let slice = &mut self.slices[self.in_use];
        let t_range = self.latest_filled_up_to..self.latest_filled_up_to + t_size_in_p_units;
        self.latest_filled_up_to += t_size_in_p_units;

        let ptr = &raw mut slice[t_range];
        ptr as *mut T
    }

    fn reset(&mut self) {
        self.in_use = 0;
        self.latest_filled_up_to = 0;
        let biggest = self.slices.pop().expect("UNREACHABLE");
        self.slices.clear();
        self.slices.push(biggest);
    }

    fn new() -> Self {
        let slice = Self::make_slice(8);
        Self {
            slices: vec![slice],
            in_use: 0,
            latest_filled_up_to: 0,
        }
    }
}

impl<'f> Alloc<'f> {
    pub fn size(&self) -> usize {
        let vecs_size = self.vecs16.size()
            + self.vecs8.size()
            + self.vecs4.size()
            + self.vecs2.size()
            + self.vecs1.size();
        let singles_size = self.singles16.size()
            + self.singles8.size()
            + self.singles4.size()
            + self.singles2.size()
            + self.singles1.size();
        vecs_size + singles_size
    }

    pub fn frame_vec<'s, T>(&'s mut self) -> FrameVec<'s, 'f, T> {
        let vec = match align_of::<T>() {
            16 => self.vecs16.get_new(),
            8 => self.vecs8.get_new(),
            4 => self.vecs4.get_new(),
            2 => self.vecs2.get_new(),
            1 => self.vecs1.get_new(),
            _ => panic!("Only alignments of 16, 8, 4, 2 and 1 bytes are supported."),
        };
        FrameVec {
            vec,
            _frame_lifetime: PhantomData,
        }
    }

    pub fn frame_val<T>(&mut self, val: T) -> &'f mut T {
        let ptr: *mut T = match align_of::<T>() {
            16 => self.singles16.get_new(),
            8 => self.singles8.get_new(),
            4 => self.singles4.get_new(),
            2 => self.singles2.get_new(),
            1 => self.singles1.get_new(),
            _ => panic!("Only alignments of 16, 8, 4, 2 and 1 bytes are supported."),
        };

        unsafe {
            ptr.write(val);
            &mut *ptr
        }
    }
}

impl<'r> Debug for Alloc<'r> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Alloc")
            .field("alloc_seq", &self.alloc_seq)
            .field("size", &(self.size()))
            .finish()
    }
}

impl Default for Alloc<'static> {
    fn default() -> Self {
        #[cfg(feature = "eprint_alloc")]
        eprintln!("Creating a new frame allocator");
        Alloc {
            alloc_seq: 1,
            _frame_lifetime: PhantomData,
            vecs16: Vecs::new(),
            vecs8: Vecs::new(),
            vecs4: Vecs::new(),
            vecs2: Vecs::new(),
            vecs1: Vecs::new(),
            singles16: Singles::new(),
            singles8: Singles::new(),
            singles4: Singles::new(),
            singles2: Singles::new(),
            singles1: Singles::new(),
        }
    }
}

impl Alloc<'static> {
    pub fn reset(&mut self, seq: usize) {
        self.alloc_seq = seq;
        self.vecs16.reset();
        self.vecs8.reset();
        self.vecs4.reset();
        self.vecs2.reset();
        self.vecs1.reset();
        self.singles16.reset();
        self.singles8.reset();
        self.singles4.reset();
        self.singles2.reset();
        self.singles1.reset();
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::{Alloc, FrameVec, Singles, Vecs};

    #[test]
    #[should_panic]
    fn test_wrong_align_single() {
        let mut single = Singles::<u8>::new();
        single.get_new::<u16>();
    }

    #[test]
    #[should_panic]
    fn test_wrong_align_vecs() {
        let mut vecs = Vecs::<u8>::new();
        vecs.get_new::<u16>();
    }

    #[test]
    #[should_panic]
    fn test_wrong_align_alloc_vec() {
        #[repr(align(32))]
        struct WeirdAlign;

        let mut alloc = Alloc::default();
        let mut vec = alloc.frame_vec();
        vec.push(WeirdAlign);
    }

    #[test]
    #[should_panic]
    fn test_wrong_align_alloc_val() {
        #[repr(align(32))]
        struct WeirdAlign;

        let mut alloc = Alloc::default();
        let _ = alloc.frame_val(WeirdAlign);
    }

    #[test]
    fn test_singles_u8() {
        let mut single = Singles::<u8>::new();
        assert_eq!(single.size(), 8); // Preallocated
        single.get_new::<u8>();
        single.get_new::<u8>();
        single.get_new::<u8>();
        single.get_new::<u8>();
        single.get_new::<u8>();
        single.get_new::<u8>();
        single.get_new::<u8>();
        single.get_new::<u8>();
        assert_eq!(single.size(), 8); // Full
        single.get_new::<u8>();
        assert_eq!(single.size(), 24); // Grew by 16
        single.get_new::<[u8; 15]>();
        assert_eq!(single.size(), 24); // Full again
        single.get_new::<u8>();
        assert_eq!(single.size(), 56); // Grew by 32
        single.get_new::<[u8; 32]>(); // Overflowing by 1
        assert_eq!(single.size(), 120); // Grew by 64
        single.get_new::<[u8; 130]>(); // Overflowing by over next increment (128)
        assert_eq!(single.size(), 380); // Grew by 2 * 130, not by 128
        single.get_new::<[u8; 130]>();
        assert_eq!(single.size(), 380); // Full again
        single.get_new::<u8>();
        assert_eq!(single.size(), 900); // Grew by 520
    }

    #[test]
    fn test_singles_u16() {
        let mut single = Singles::<u16>::new();
        assert_eq!(single.size(), 16); // Preallocated
        single.get_new::<u16>();
        single.get_new::<u16>();
        single.get_new::<u16>();
        single.get_new::<u16>();
        single.get_new::<u16>();
        single.get_new::<u16>();
        single.get_new::<u16>();
        single.get_new::<u16>();
        assert_eq!(single.size(), 16); // Full
        single.get_new::<u16>();
        assert_eq!(single.size(), 48); // Grew by 32
        single.get_new::<[u16; 15]>();
        assert_eq!(single.size(), 48); // Full again
        single.get_new::<u16>();
        assert_eq!(single.size(), 112); // Grew by 64
        single.get_new::<[u16; 32]>(); // Overflowing by 2
        assert_eq!(single.size(), 240); // Grew by 128
        single.get_new::<[u16; 130]>(); // Overflowing by over next increment (256)
        assert_eq!(single.size(), 760); // Grew by 2 * 260, not by 256
        single.get_new::<[u16; 130]>();
        assert_eq!(single.size(), 760); // Full again
        single.get_new::<u16>();
        assert_eq!(single.size(), 1800); // Grew by 1040
    }

    #[test]
    fn test_singles_reset() {
        let mut single = Singles::<u16>::new();
        assert_eq!(single.size(), 16); // Preallocated
        single.get_new::<[u16; 50]>();
        assert_eq!(single.size(), 216);

        single.reset();

        assert_eq!(single.size(), 200); // The other than the bigger slice are dropped
        single.get_new::<[u16; 100]>(); // Would overflow a 16-byte slice
        assert_eq!(single.size(), 200); // But it fits this!
    }

    #[test]
    fn test_vecs() {
        let mut vecs = Vecs::<u8>::new();
        assert_eq!(vecs.size(), 0); // No prealloc
        vecs.get_new::<u8>().push(1);
        assert_eq!(vecs.size(), 8);
        vecs.get_new::<u8>();
        vecs.get_new::<u8>();
        assert_eq!(vecs.size(), 8);
        let a = vecs.get_new::<u8>();
        a.push(1);
        assert_eq!(vecs.size(), 16);
    }

    #[test]
    fn test_vecs_reset() {
        let mut vecs = Vecs::<u8>::new();
        assert_eq!(vecs.size(), 0);
        vecs.get_new::<u8>();
        vecs.get_new::<u8>().push(1);
        assert_eq!(vecs.size(), 8);

        vecs.reset();

        assert_eq!(vecs.size(), 8); // Nothing gets deallocated
        let a = vecs.get_new::<u8>();
        assert_eq!(*a, []); // The 0th Vec is empty (of course)
        let b = vecs.get_new::<u8>();
        assert_eq!(*b, []); // The 1th Vec is cleared
        b.push(1);
        assert_eq!(vecs.size(), 8); // The 1th Vec has capacity of at least 1, so no allocations

        vecs.reset();

        vecs.get_new::<u8>().push(2);
        vecs.get_new::<u8>();
        assert_eq!(vecs.size(), 16); // Both 0th and 1th have now allocations
    }

    #[test]
    fn test_frame_vec() {
        fn helper<T: Debug + Eq>(mut vec: FrameVec<T>, iter: impl Iterator<Item = T> + Clone) {
            assert_eq!(*vec, &[]);
            let result = iter.clone();
            for i in iter {
                vec.push(i.into());
            }
            assert_eq!(vec.into_slice(), result.collect::<Vec<_>>().as_slice());
        }
        let mut alloc = Alloc::default();

        let orig_size = alloc.size();
        let vec: FrameVec<u8> = alloc.frame_vec();
        helper(vec, 0..20);
        assert_eq!(alloc.size(), orig_size + 32);

        let orig_size = alloc.size();
        let vec: FrameVec<u16> = alloc.frame_vec();
        helper(vec, 20..40);
        assert_eq!(alloc.size(), orig_size + 64);

        let vec: FrameVec<u32> = alloc.frame_vec();
        helper(vec, 40..60);

        let vec: FrameVec<u64> = alloc.frame_vec();
        helper(vec, 60..80);

        let vec: FrameVec<u128> = alloc.frame_vec();
        helper(vec, 80..100);

        for _ in 0..30 {
            let vec: FrameVec<u32> = alloc.frame_vec();
            helper(vec, 100..120);
        }

        for _ in 0..30 {
            let vec: FrameVec<u64> = alloc.frame_vec();
            helper(vec, 120..140);
        }

        alloc.reset(2);

        let vec: FrameVec<(u64, u64)> = alloc.frame_vec();
        helper(vec, (140..160).map(|i| (i, i + 1)));

        let vec: FrameVec<(u64, u8)> = alloc.frame_vec();
        helper(vec, (160..180).map(|i| (i, i as u8 + 1)));

        for _ in 0..30 {
            let vec: FrameVec<u16> = alloc.frame_vec();
            helper(vec, 180..200);
        }

        for _ in 0..30 {
            let vec: FrameVec<u32> = alloc.frame_vec();
            helper(vec, 200..220);
        }

        for _ in 0..30 {
            let vec: FrameVec<u64> = alloc.frame_vec();
            helper(vec, 220..240);
        }
    }

    #[test]
    fn test_frame_vec_drop() {
        let mut vecs = Vecs::<u32>::new();
        let _: &mut Vec<(u32, u32)> = vecs.get_new();
        drop(vecs);
    }

    #[test]
    fn test_alloc_debug() {
        let mut alloc = Alloc::default();
        // 248 = default
        assert_eq!(format!("{:?}", alloc), "Alloc { alloc_seq: 1, size: 248 }");

        alloc.reset(2);
        assert_eq!(format!("{:?}", alloc), "Alloc { alloc_seq: 2, size: 248 }");

        alloc.frame_val(0_u32); // Fits in prealloc, so doesn't affect size

        assert_eq!(format!("{:?}", alloc), "Alloc { alloc_seq: 2, size: 248 }");

        let bytes_per = size_of::<u32>();

        let mut vec = alloc.frame_vec();
        vec.push(69_u32);
        assert_eq!(alloc.vecs4.vecs[0].1.capacity(), bytes_per);

        let mut test_vec = Vec::new();
        test_vec.push(69_u32);
        assert_eq!(test_vec.capacity(), bytes_per);

        assert_eq!(alloc.size(), 248 + test_vec.capacity() * bytes_per);

        assert_eq!(
            format!("{:?}", alloc),
            format!(
                "Alloc {{ alloc_seq: 2, size: {} }}",
                248 + test_vec.capacity() * bytes_per
            ),
        );
    }

    #[test]
    fn test_frame_val() {
        let mut alloc = Alloc::default();

        for i in 0..20_u8 {
            assert_eq!(*alloc.frame_val(i), i);
        }
        for i in 20..40_u16 {
            assert_eq!(*alloc.frame_val(i), i);
        }
        for i in 40..60_u32 {
            assert_eq!(*alloc.frame_val(i), i);
        }
        for i in 60..80_u64 {
            assert_eq!(*alloc.frame_val(i), i);
        }
        for i in 80..100_u128 {
            assert_eq!(*alloc.frame_val(i), i);
        }
        for i in (100..120_u32).map(|i| (i, i + 1)) {
            assert_eq!(*alloc.frame_val(i), i);
        }
        for i in (120..140_u64).map(|i| (i, i + 1)) {
            assert_eq!(*alloc.frame_val(i), i);
        }

        alloc.reset(2);

        for i in (100..120_u32).map(|i| (i, i + 1)) {
            assert_eq!(*alloc.frame_val(i), i);
        }

        for i in 40..60_u32 {
            assert_eq!(*alloc.frame_val(i), i);
        }

        for i in 20..40_u16 {
            assert_eq!(*alloc.frame_val(i), i);
        }

        for i in 0..20_u8 {
            assert_eq!(*alloc.frame_val(i), i);
        }
    }
}
