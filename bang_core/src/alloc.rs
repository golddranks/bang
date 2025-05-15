use arena::Sink;
pub use arena::{Arena, ArenaGuard, Id, Managed};

#[repr(C)]
pub struct Mem<'frame> {
    pub alloc_seq: usize,
    pub arena: &'frame mut ArenaGuard<'frame>,
}

impl<'f> Mem<'f> {
    pub fn new(arena: &'f mut ArenaGuard<'f>) -> Self {
        Mem {
            alloc_seq: 0,
            arena,
        }
    }

    pub fn vec<T>(&mut self) -> &'f mut Vec<T> {
        self.arena.alloc_vec()
    }

    pub fn val<T>(&mut self, val: T) -> &'f mut T {
        self.arena.alloc_val(val)
    }

    pub fn slice<T>(&mut self, slice: &[T]) -> &'f mut [T] {
        self.arena.alloc_slice(slice)
    }

    pub fn str<T>(&mut self, str: &str) -> &'f mut str {
        self.arena.alloc_str(str)
    }

    pub fn string<T>(&mut self, str: &str) -> &'f mut String {
        self.arena.alloc_string(str)
    }

    pub fn from_iter<T, I>(&mut self, iter: I) -> &'f mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.arena.alloc_iter(iter.into_iter())
    }

    pub fn sink<'s, T>(&'s mut self) -> Sink<'s, 'f, T> {
        self.arena.alloc_sink()
    }
}

#[cfg(test)]
mod tests {
    use arena::Arena;

    use super::*;

    #[test]
    fn test_mem() {
        let mut arena = Arena::default();
        let ag = arena.fresh_arena(1);
        let mut mem = Mem::new(ag);

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
