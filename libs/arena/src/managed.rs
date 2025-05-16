use std::{
    any::type_name,
    collections::VecDeque,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicPtr, AtomicU16, AtomicU32, AtomicU64, Ordering},
};

static GLOBAL_ARENA_SEQ: AtomicU16 = AtomicU16::new(0);

use crate::Erased;

struct Store<T> {
    alloc_seq: usize,
    store: Vec<MaybeUninit<T>>,
    generations: Vec<u32>,
}

pub struct Managed<'l, T, I = Erased> {
    arena_id: u16,
    stores: VecDeque<Store<T>>,
    current_store: AtomicPtr<Store<T>>,
    alloc_seq: &'l AtomicU64,
    free_list: Vec<u32>,
    reap_list: Vec<u32>,
    _basetype_marker: PhantomData<T>,
    _iface_marker: PhantomData<I>,
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
        let (arena_id, idx, gener) = self.parts();
        let type_name = type_name::<T>();
        write!(
            f,
            "Id(arena_id={arena_id}, idx={idx}, gen={gener}, type={type_name})",
        )
    }
}

impl<T> Id<T> {
    fn new(arena_id: u16, idx: usize, generation: u32) -> Self {
        debug_assert!(idx <= 0xFFFFFF);
        debug_assert!(generation <= 0xFFFFFF);
        let id = (arena_id as u64) << 48 | (idx as u64) << 24 | generation as u64;
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn arena_id(self) -> u16 {
        (self.id >> 48) as u16
    }

    pub fn idx(self) -> usize {
        ((self.id >> 24) & 0xFFFFFF) as usize
    }

    pub fn gener(self) -> u32 {
        (self.id & 0xFFFFFF) as u32
    }

    pub fn parts(self) -> (u16, usize, u32) {
        (self.arena_id(), self.idx(), self.gener())
    }
}

impl<'l, T, I> Managed<'l, T, I>
where
    MaybeUninit<T>: Clone,
{
    fn new(seq: usize) -> Self {
        let arena_id = GLOBAL_ARENA_SEQ.fetch_add(1, Ordering::Relaxed);
        let mut stores = VecDeque::new();
        stores.push_front(Store {
            alloc_seq: 1,
            store: Vec::new(),
            generations: Vec::new(),
        });
        // UNREACHABLE: `self.stores` was just initialized with one `Vec`
        let current_store = AtomicPtr::new(stores.front_mut().expect("UNREACHABLE: never empty"));
        Self {
            arena_id,
            stores,
            current_store,
            alloc_seq: 1,
            free_list: Vec::new(),
            reap_list: Vec::new(),
            _iface_marker: PhantomData,
            _basetype_marker: PhantomData,
        }
    }

    fn current_store(&mut self) -> &mut Store<T> {
        // Safety: TODO
        unsafe { &mut *self.current_store.load(Ordering::Acquire) }
    }

    fn grow(&mut self) -> &mut Store<T> {
        let full_store = self.current_store();
        let mut store = Vec::with_capacity(full_store.store.capacity() * 2);
        let mut generations = Vec::with_capacity(store.len());
        store.extend(full_store.store.iter().cloned());
        generations.extend(full_store.generations.iter().cloned());
        let new_store = Store {
            alloc_seq: self.shared.alloc_seq.load(),
            store,
            generations,
        };
        self.stores.push_front(new_store);
        // UNREACHABLE: we just added an item, so `self.stores` is never empty
        self.stores.front_mut().expect("UNREACHABLE: never empty")
    }

    pub fn alloc(&mut self, val: T) -> Id<T> {
        let arena_id = self.arena_id;
        if self.free_list.is_empty() {
            let current_store = self.current_store();
            let next_idx = current_store.store.len();
            assert!(next_idx < 0xFFFFFF);
            let current_store = if next_idx < current_store.store.capacity() {
                current_store
            } else {
                self.grow()
            };
            current_store.store.push(MaybeUninit::new(val));
            current_store.generations.push(0);
            Id::new(arena_id, next_idx, 0)
        } else {
            // UNREACHABLE: self.free_list was checked not to be empty in the
            // if condition that led to this else branch
            let idx = self
                .free_list
                .pop()
                .expect("UNREACHABLE: checked not to be empty") as usize;
            let current_store = self.current_store();
            current_store.store[idx] = MaybeUninit::new(val);
            let gener = &mut current_store.generations[idx];
            *gener += 1;
            Id::new(arena_id, idx, *gener)
        }
    }

    pub fn alloc_upcast(&mut self, val: T) -> Id<I> {
        Self::upcast(self.alloc(val))
    }

    pub fn reap_deferred_now(&mut self) {
        self.reap_list.sort_unstable();
        self.reap_list.dedup();
        for idx in self.reap_list.drain(..) {
            // Safety: Only valid indices of currently live values are inserted to
            // `reap_list`. Other methods don't insert anything. The list is drained
            // only by `reap_deferred_now`, and possible duplicates are deduplicated,
            // so drop and freeing procedure is called exactly once per value.
            unsafe { self.stores[idx as usize].assume_init_drop() };
            self.generations[idx as usize] += 1;
            self.free_list.push(idx);
        }
    }

    /// Marks the value associated with the given ID to be freed next time
    /// `reap_deferred_now` is called.
    ///
    /// # Panics
    ///
    /// This function will panic if the ID points to an invalid index.
    /// If you use an ID retrieved from this arena, you will never encounter
    /// this case, but cross-using ID's between two separate arenas is a bug
    /// and may panic. This is considered API misuse and is on caller's
    /// responsibility.
    pub fn defer_free(&mut self, id: Id<T>) -> bool {
        let (arena_id, idx, gener) = id.parts();
        // Panic: documented in the docstring. Caller's responsibility.
        assert!(arena_id == self.arena_id);
        assert!(idx < self.generations.len());
        if gener < self.generations[idx] {
            eprintln!("Warning: trying to free already freed entity {id:?}");
            false
        } else {
            self.reap_list.push(idx as u32);
            true
        }
    }

    /// Retrieves a reference to the value associated with the given ID.
    /// Returns `None` if the value has been freed.
    ///
    /// # Panics
    ///
    /// This function will panic if the ID points to an invalid index.
    /// If you use an ID retrieved from this arena, you will never encounter
    /// this case, but cross-using ID's between two separate arenas is a bug
    /// and may panic. This is considered API misuse and is on caller's
    /// responsibility.
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        let (arena_id, idx, gener) = id.parts();
        // Panic: documented in the docstring. Caller's responsibility.
        assert!(arena_id == self.arena_id);
        assert!(idx < self.generations.len());
        if gener == self.generations[idx] {
            // Safety: if the generation matches, the value is guaranteed to be
            // in an initialized and valid state.
            Some(unsafe { self.stores[idx].assume_init_ref() })
        } else {
            None
        }
    }

    /// Retrieves a mutable reference to the value associated with the given ID.
    /// Returns `None` if the value has been freed.
    ///
    /// # Panics
    ///
    /// This function will panic if the ID points to an invalid index.
    /// If you use an ID retrieved from this arena, you will never encounter
    /// this case, but cross-using ID's between two separate arenas is a bug
    /// and may panic. This is considered API misuse and is on caller's
    /// responsibility.
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        let (arena_id, idx, gener) = id.parts();
        // Panic: documented in the docstring. Caller's responsibility.
        assert!(arena_id == self.arena_id);
        assert!(idx < self.generations.len());
        if gener == self.generations[idx] {
            // Safety: if the generation matches, the value is guaranteed to be
            // in an initialized and valid state.
            Some(unsafe { self.stores[idx].assume_init_mut() })
        } else {
            None
        }
    }

    /// Checks if given index contains a valid, living object.
    pub fn is_idx_live(&self, idx: usize) -> bool {
        self.generations
            .get(idx)
            .map(|gener| gener & 1 == 0)
            .unwrap_or(false)
    }

    pub fn upcast(id: Id<T>) -> Id<I> {
        Id {
            id: id.id,
            _marker: PhantomData,
        }
    }

    pub fn downcast(id: Id<I>) -> Id<T> {
        Id {
            id: id.id,
            _marker: PhantomData,
        }
    }
}

impl<T, I> Drop for Managed<T, I> {
    fn drop(&mut self) {
        for idx in 0..self.stores.len() {
            if self.is_idx_live(idx) {
                // Safety: `is_idx_live` examines the generation, and guarantees
                // the value is in an initialized and valid state.
                unsafe { self.stores[idx].assume_init_drop() };
            }
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
        let mut managed = Managed::<_, Erased>::new(1);

        assert_eq!(managed.is_idx_live(0), false);
        assert_eq!(managed.is_idx_live(1), false);
        assert_eq!(managed.is_idx_live(2), false);
        let id = managed.alloc(5);
        assert_eq!(managed.is_idx_live(id.idx()), true);
        assert_eq!(managed.get(id), Some(&5));
        assert_eq!(managed.get_mut(id), Some(&mut 5));
        assert_eq!(managed.defer_free(id), true);
        assert_eq!(managed.get(id), Some(&5));
        assert_eq!(managed.is_idx_live(id.idx()), true);
        managed.reap_deferred_now();
        assert_eq!(managed.is_idx_live(id.idx()), false);
        assert_eq!(managed.defer_free(id), false);
        assert_eq!(managed.get(id), None);
        assert_eq!(managed.get_mut(id), None);
        let id2 = managed.alloc(6);
        let id3 = managed.alloc(7);
        assert_eq!(managed.is_idx_live(id.idx()), true); // Because the slot is being reused
        assert_eq!(managed.is_idx_live(id2.idx()), true);
        assert_eq!(managed.is_idx_live(id3.idx()), true);
        assert_eq!(managed.get(id2), Some(&6));
        assert_ne!(id, id2);
        assert_eq!(managed.is_idx_live(0), true);
        assert_eq!(managed.is_idx_live(1), true);
        assert_eq!(managed.is_idx_live(2), false);
    }

    #[test]
    fn test_id() {
        struct Dummy;
        let id: Id<Dummy> = Id::new(1, 3, 5);
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

    #[test]
    fn managed_drop_on_free() {
        let mut managed = Managed::<_, Erased>::new(1);
        let id = managed.alloc(String::from("Hello, World!"));
        managed.defer_free(id);
        managed.reap_deferred_now();
    }

    #[test]
    fn managed_drop_on_container_drop() {
        let mut managed = Managed::<_, Erased>::new(1);
        let _ = managed.alloc(String::from("Hello, World!"));
        drop(managed)
    }
}
