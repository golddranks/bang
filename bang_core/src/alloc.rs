use arena::Arena;

#[repr(C)]
pub struct Mem<'frame> {
    pub alloc_seq: usize,
    pub arena: &'frame mut Arena<'frame>,
}

impl<'f> Mem<'f> {
    pub fn new(arena: &'f mut Arena<'f>) -> Self {
        Mem {
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

    pub fn slice<T>(&mut self, slice: &[T]) -> &'f mut [T] {
        self.arena.allocate_slice(slice)
    }

    pub fn from_iter<T, I>(&mut self, iter: I) -> &'f mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.arena.allocate_iter(iter.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use arena::ArenaContainer;

    use super::*;

    #[test]
    fn test_mem() {
        let mut container = ArenaContainer::default();
        let arena = container.new_arena(1);
        let mut mem = Mem::new(arena);

        let vec = mem.vec();
        vec.push(69);
        let val = mem.val(420);
        let slice = mem.slice(&[1, 2, 3]);
        let slice_from_iter = mem.from_iter([4, 5, 6]);

        assert_eq!(vec, &[69]);
        assert_eq!(*val, 420);
        assert_eq!(slice, &[1, 2, 3]);
        assert_eq!(slice_from_iter, &[4, 5, 6]);
    }
}
