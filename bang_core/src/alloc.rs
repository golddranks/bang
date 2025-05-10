use arena::Arena;

#[repr(C)]
pub struct Alloc<'frame> {
    pub alloc_seq: usize,
    pub arena: &'frame mut Arena<'frame>,
}

impl<'f> Alloc<'f> {
    pub fn new(arena: &'f mut Arena<'f>) -> Self {
        Alloc {
            alloc_seq: 0,
            arena,
        }
    }

    pub fn vec<T>(&mut self) -> &'f mut Vec<T> {
        self.arena.allocate_vec()
    }

    pub fn val<T>(&mut self, val: T) -> &'f mut T {
        self.arena.allocate_val(val)
    }
}
/*
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
 */
