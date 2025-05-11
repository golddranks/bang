use std::{any::type_name, fmt::Debug, hash::Hash, marker::PhantomData, mem::MaybeUninit};

pub struct Managed<T> {
    store: Vec<MaybeUninit<T>>,
    generations: Vec<u32>,
    free_list: Vec<u32>,
    reap_list: Vec<u32>,
}

pub struct Id<T> {
    id: u64,
    _marker: PhantomData<T>,
}

// Manually implement common traits because derive doesn't work in presence
// of a generic PhantomData

impl<T> Copy for Id<T> {}
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Id<T> {}
impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}
impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (idx, gener) = self.parts();
        write!(
            f,
            "Id(idx={}, gen={}, type={})",
            idx,
            gener,
            type_name::<T>()
        )
    }
}

impl<T> Id<T> {
    pub fn new(idx: usize, generation: u32) -> Self {
        let id = (idx as u64) << 32 | generation as u64;
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn parts(self) -> (usize, u32) {
        let idx = (self.id >> 32) as usize;
        let gener = (self.id & 0x0000_0000_FFFF_FFFF) as u32;
        (idx, gener)
    }
}

impl<T> Managed<T> {
    pub fn new() -> Self {
        Self {
            store: Vec::new(),
            generations: Vec::new(),
            free_list: Vec::new(),
            reap_list: Vec::new(),
        }
    }

    pub fn alloc(&mut self, val: T) -> Id<T> {
        let (idx, gener) = if self.free_list.is_empty() {
            let idx = self.store.len();
            self.store.push(MaybeUninit::new(val));
            self.generations.push(0);
            (idx, 0)
        } else {
            let idx = self.free_list.pop().unwrap() as usize;
            let gener = &mut self.generations[idx];
            *gener += 1;
            self.store[idx] = MaybeUninit::new(val);
            (idx, *gener)
        };
        Id::new(idx, gener)
    }

    pub fn reap_deferred(&mut self) {
        for idx in self.reap_list.drain(..) {
            self.generations[idx as usize] += 1;
            self.free_list.push(idx);
        }
    }

    pub fn defer_free(&mut self, id: Id<T>) -> bool {
        let (idx, gener) = id.parts();
        assert!(idx < self.store.len());
        if gener < self.generations[idx] {
            eprintln!("Warning: trying to free already freed entity {:?}", id);
            false
        } else {
            self.reap_list.push(idx as u32);
            true
        }
    }

    pub fn get(&self, id: Id<T>) -> Option<&T> {
        let (idx, gener) = id.parts();
        assert!(idx < self.store.len());
        if gener == self.generations[idx] {
            Some(unsafe { self.store[idx].assume_init_ref() })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        let (idx, gener) = id.parts();
        assert!(idx < self.store.len());
        if gener == self.generations[idx] {
            Some(unsafe { self.store[idx].assume_init_mut() })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cmp::Ordering,
        hash::{DefaultHasher, Hasher},
    };

    #[test]
    fn test_managed() {
        let mut managed = Managed::new();

        let id = managed.alloc(5);
        assert_eq!(managed.get(id), Some(&5));
        assert_eq!(managed.get_mut(id), Some(&mut 5));
        assert_eq!(managed.defer_free(id), true);
        assert_eq!(managed.get(id), Some(&5));
        managed.reap_deferred();
        assert_eq!(managed.defer_free(id), false);
        assert_eq!(managed.get(id), None);
        assert_eq!(managed.get_mut(id), None);
        let id2 = managed.alloc(6);
        assert_eq!(managed.get(id2), Some(&6));
        assert_ne!(id, id2);
    }

    #[test]
    fn test_id() {
        struct Dummy;
        let id: Id<Dummy> = Id::new(3, 5);
        let _id2 = id.clone();
        let _id3 = id; // using Copy trait
        assert!(id.eq(&_id2)); // Using Eq trait
        assert_eq!(id.cmp(&_id2), Ordering::Equal); // Using Ord trait
        assert_eq!(id.partial_cmp(&_id2), Some(Ordering::Equal)); // Using PartialOrd trait
        let mut state = DefaultHasher::new();
        id.hash(&mut state);
        let mut state2 = DefaultHasher::new();
        id.id.hash(&mut state2);
        assert_eq!(state.finish(), state2.finish());
        assert_eq!(
            format!("{id:?}"),
            "Id(idx=3, gen=5, type=arena::managed::tests::test_id::Dummy)"
        );
    }
}
