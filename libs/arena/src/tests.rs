use super::*;
use std::ptr::addr_eq;

#[test]
fn test_no_allocation() {
    let mut alloc = Arena::default();
    let _arena = alloc.fresh_arena(1);
}

#[test]
fn test_empty_vec_allocation() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);
    let _vec = arena.alloc_vec::<u32>();
}

#[test]
fn test_basic_vec_allocation() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec = arena.alloc_vec::<u32>();
    assert!(vec.is_empty());

    vec.push(42);
    vec.push(100);

    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], 42);
    assert_eq!(vec[1], 100);
}

#[test]
fn test_basic_val_allocation() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let val1 = arena.alloc_val(42u32);
    let val2 = arena.alloc_val(100u32);

    assert_eq!(*val1, 42);
    assert_eq!(*val2, 100);

    *val1 = 99;
    assert_eq!(*val1, 99);
    assert_eq!(*val2, 100);
}

#[test]
fn test_same_vec_layout() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec1 = arena.alloc_vec::<u8>();
    let vec2 = arena.alloc_vec::<u8>();

    vec1.extend_from_slice(&[1, 2, 3]);
    vec2.extend_from_slice(&[4, 5, 6]);

    assert_eq!(vec1, &[1, 2, 3]);
    assert_eq!(vec2, &[4, 5, 6]);
}

#[test]
fn test_multiple_vec_allocations() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec1 = arena.alloc_vec::<u8>();
    let vec2 = arena.alloc_vec::<u16>();
    let vec3 = arena.alloc_vec::<u32>();
    let vec4 = arena.alloc_vec::<u64>();
    let vec5 = arena.alloc_vec::<u8>();

    vec1.extend_from_slice(&[1, 2, 3]);
    vec2.extend_from_slice(&[10, 20, 30]);
    vec3.extend_from_slice(&[100, 200, 300]);
    vec4.extend_from_slice(&[1000, 2000, 3000]);
    vec5.extend_from_slice(&[4, 5, 6]);

    assert_eq!(vec1, &[1, 2, 3]);
    assert_eq!(vec2, &[10, 20, 30]);
    assert_eq!(vec3, &[100, 200, 300]);
    assert_eq!(vec4, &[1000, 2000, 3000]);
    assert_eq!(vec5, &[4, 5, 6]);
}

#[test]
fn test_different_vec_alignments() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec1 = arena.alloc_vec::<u8>();
    let vec2 = arena.alloc_vec::<u16>();
    let vec4 = arena.alloc_vec::<u32>();
    let vec8 = arena.alloc_vec::<u64>();
    let vec16 = arena.alloc_vec::<u128>();
    let vec8_8 = arena.alloc_vec::<[u64; 2]>();

    vec1.push(1);
    vec2.push(2);
    vec4.push(4);
    vec8.push(8);
    vec16.push(16);
    vec8_8.push([16, 16]);

    assert_eq!(vec1[0], 1);
    assert_eq!(vec2[0], 2);
    assert_eq!(vec4[0], 4);
    assert_eq!(vec8[0], 8);
    assert_eq!(vec16[0], 16);
    assert_eq!(vec8_8[0], [16, 16]);
}

#[test]
fn test_different_val_alignments() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let val1 = arena.alloc_val(0x42_u8);
    let val2 = arena.alloc_val(0x4243_u16);
    let val3 = arena.alloc_val(0x42434445_u32);
    let val4 = arena.alloc_val(0x4243444546474849_u64);
    let val5 = arena.alloc_val(0x42434445464748495051525354555657_u128);

    assert_eq!(*val1, 0x42u8);
    assert_eq!(*val2, 0x4243u16);
    assert_eq!(*val3, 0x42434445u32);
    assert_eq!(*val4, 0x4243444546474849u64);
    assert_eq!(*val5, 0x42434445464748495051525354555657_u128);

    let mut usage = MemoryUsage::default();
    arena.val_memory_usage(&mut usage);

    let expected_content = size_of::<u8>()
        + size_of::<u16>()
        + size_of::<u32>()
        + size_of::<u64>()
        + size_of::<u128>();
    assert_eq!(usage.content_bytes, expected_content);

    *val1 = 0x11;
    *val2 = 0x2222;
    *val3 = 0x33333333;
    *val4 = 0x4444444444444444;
    *val5 = 0x55555555555555555555555555555555;

    assert_eq!(*val1, 0x11);
    assert_eq!(*val2, 0x2222);
    assert_eq!(*val3, 0x33333333);
    assert_eq!(*val4, 0x4444444444444444);
    assert_eq!(*val5, 0x55555555555555555555555555555555);

    let mut usage = MemoryUsage::default();
    arena.val_memory_usage(&mut usage);
    assert_eq!(usage.content_bytes, expected_content);
}

#[test]
fn test_vec_reset() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec = arena.alloc_vec::<u32>();
    vec.push(42);
    assert_eq!(vec.len(), 1);

    let arena = alloc.fresh_arena(2);
    let vec = arena.alloc_vec::<u32>();
    assert!(vec.is_empty());
}

#[test]
fn test_val_reset() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let val1 = arena.alloc_val(42u32);
    let val2 = arena.alloc_val(43u64);
    assert_eq!(*val1, 42u32);
    assert_eq!(*val2, 43u64);

    let arena = alloc.fresh_arena(2);

    let new_val1 = arena.alloc_val(100u32);
    let new_val2 = arena.alloc_val(200u64);
    assert_eq!(*new_val1, 100u32);
    assert_eq!(*new_val2, 200u64);
}

#[test]
fn test_large_vec_allocation() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec = arena.alloc_vec::<u32>();
    for i in 0..1000 {
        vec.push(i);
    }

    for i in 0..1000 {
        assert_eq!(vec[i], i as u32);
    }

    let usage = alloc.memory_usage();
    let expected_content = 1000 * size_of::<u32>();
    assert_eq!(usage.content_bytes, expected_content);
}

#[test]
fn test_large_val_allocation() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let large_value = arena.alloc_val([42u64; 32]);

    assert_eq!(large_value[0], 42);
    assert_eq!(large_value[31], 42);

    large_value[10] = 100;
    large_value[20] = 200;

    assert_eq!(large_value[10], 100);
    assert_eq!(large_value[20], 200);

    let large_value2 = arena.alloc_val([84u64; 32]);

    assert_eq!(large_value[10], 100);
    assert_eq!(large_value[20], 200);
    assert_eq!(large_value2[10], 84);
    assert_eq!(large_value2[20], 84);

    large_value2[10] = 1010;
    large_value2[20] = 2020;

    assert_eq!(large_value[10], 100);
    assert_eq!(large_value[20], 200);
    assert_eq!(large_value2[10], 1010);
    assert_eq!(large_value2[20], 2020);

    let usage = alloc.memory_usage();
    let expected_content = 2 * 32 * size_of::<u64>();
    assert_eq!(usage.content_bytes, expected_content);
}

#[test]
fn test_mixed_vec_and_val_allocation() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec1 = arena.alloc_vec::<u32>();
    let val1 = arena.alloc_val(42u64);
    let vec2 = arena.alloc_vec::<u8>();
    let val2 = arena.alloc_val(43u16);

    vec1.push(100);
    vec1.push(200);
    vec2.push(10);
    vec2.push(20);
    vec2.push(30);

    assert_eq!(vec1.len(), 2);
    assert_eq!(vec1[0], 100);
    assert_eq!(vec1[1], 200);

    assert_eq!(vec2.len(), 3);
    assert_eq!(vec2[0], 10);
    assert_eq!(vec2[1], 20);
    assert_eq!(vec2[2], 30);

    assert_eq!(*val1, 42u64);
    assert_eq!(*val2, 43u16);

    let usage = unsafe { arena.memory_usage() };
    let expected_content =
        size_of::<u64>() + size_of::<u16>() + (2 * size_of::<u32>()) + (3 * size_of::<u8>());
    assert_eq!(usage.content_bytes, expected_content);

    *val1 = 99;
    *val2 = 999;

    assert_eq!(*val1, 99u64); // Miri: No UB
    assert_eq!(*val2, 999u16);
}

#[test]
fn test_zero_sized_val() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let empty1 = arena.alloc_val(());
    let empty2 = arena.alloc_val(());

    assert!(addr_eq(empty1, empty2));
}

#[test]
fn test_zero_sized_vec() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vec = arena.alloc_vec::<()>();

    assert_eq!(vec.capacity(), usize::MAX);
    assert_eq!(vec.len(), 0);

    vec.push(());

    assert_eq!(vec.capacity(), usize::MAX);
    assert_eq!(vec.len(), 1);
}

#[test]
fn test_val_memory_usage() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    // Check initial memory usage
    let mut empty_usage = MemoryUsage::default();
    arena.val_memory_usage(&mut empty_usage);
    assert_eq!(empty_usage.content_bytes, 0);
    assert_eq!(empty_usage.capacity_bytes, 80);

    // Allocate small values
    let _val1 = arena.alloc_val(1u8);
    let _val2 = arena.alloc_val(2u16);
    let _val3 = arena.alloc_val(3u32);

    // Check memory usage after small allocations
    let mut small_alloc_usage = MemoryUsage::default();
    arena.val_memory_usage(&mut small_alloc_usage);
    assert!(small_alloc_usage.capacity_bytes > 0);

    // Calculate exact content bytes for small allocations
    let expected_small_content = size_of::<u8>() + size_of::<u16>() + size_of::<u32>();
    assert_eq!(small_alloc_usage.content_bytes, expected_small_content);
    assert!(small_alloc_usage.total_bytes() == empty_usage.total_bytes());

    // Allocate a large array
    let _large_val = arena.alloc_val([42u64; 64]); // 512 bytes

    // Check memory usage after large allocation
    let mut large_alloc_usage = MemoryUsage::default();
    arena.val_memory_usage(&mut large_alloc_usage);
    assert!(large_alloc_usage.capacity_bytes > small_alloc_usage.capacity_bytes);

    // Calculate exact content bytes after large allocation
    let expected_large_content = expected_small_content + 64 * size_of::<u64>();
    assert_eq!(large_alloc_usage.content_bytes, expected_large_content);
    assert!(large_alloc_usage.total_bytes() > small_alloc_usage.total_bytes());

    // Create a new arena with allocator to test reset
    let arena1 = alloc.fresh_arena(2);

    // Make some allocations
    let _v1 = arena1.alloc_val(100u32);
    let _v2 = arena1.alloc_val(200u64);

    let usage1 = alloc.memory_usage();
    let expected_content1 = size_of::<u32>() + size_of::<u64>();
    assert_eq!(usage1.content_bytes, expected_content1);

    // Get a new arena (which resets the allocations)
    let arena2 = alloc.fresh_arena(3);

    // Allocate again
    let _v3 = arena2.alloc_val(300u32);

    let usage2 = alloc.memory_usage();
    let expected_content2 = size_of::<u32>();
    assert_eq!(usage2.content_bytes, expected_content2);

    // Total memory should be the same after reset and reallocation
    // (since we're reusing memory)
    assert!(usage2.total_bytes() == usage1.total_bytes());
}

#[test]
fn test_multiple_vectors_same_type() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let vectors: Vec<_> = (0..10)
        .map(|i| {
            let vec = arena.alloc_vec::<u32>();
            vec.push(i);
            vec
        })
        .collect();

    for (i, vec) in vectors.iter().enumerate() {
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], i as u32);
    }
}

#[test]
fn test_vector_capacity_reuse() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    // Grow u32 vecs (4-byte alignment)
    let vec_u32_a = arena.alloc_vec::<u32>();
    let vec_u32_b = arena.alloc_vec::<u32>();
    for i in 0..1000 {
        vec_u32_a.push(i);
    }
    for i in 0..500 {
        vec_u32_b.push(i);
    }

    let arena = alloc.fresh_arena(2);
    let vec_u32 = arena.alloc_vec::<u32>();
    let vec_f32 = arena.alloc_vec::<f32>(); // f32 also has 4-byte alignment
    assert!(
        vec_u32.capacity() >= 1000,
        "u32 vector capacity should be reused"
    );
    assert!(
        vec_f32.capacity() >= 500,
        "Vectors with same alignment should share capacity pool"
    );
}

#[test]
fn test_vec_memory_usage() {
    let mut alloc = Arena::default();
    let initial_usage = alloc.memory_usage();
    let arena = alloc.fresh_arena(1);

    assert_eq!(initial_usage.content_bytes, 0);
    assert_eq!(initial_usage.capacity_bytes, 5 * 4 * 4);

    let vec1 = arena.alloc_vec::<u8>();
    vec1.extend_from_slice(&[1, 2, 3, 4, 5]);
    let _vec1_2 = arena.alloc_vec::<u8>();
    let _vec2 = arena.alloc_vec::<u32>();
    let _vec3 = arena.alloc_vec::<u64>();
    let _vec3_2 = arena.alloc_vec::<(u64, u64)>();

    let after_alloc = alloc.memory_usage();
    assert!(after_alloc.capacity_bytes > 0);
    // Content bytes should be exactly 5 bytes (for vec1)
    assert_eq!(after_alloc.content_bytes, 5);

    let arena = alloc.fresh_arena(2);
    let vec1 = arena.alloc_vec::<u8>();
    let vec2 = arena.alloc_vec::<u32>();
    let vec3 = arena.alloc_vec::<u64>();

    vec1.extend_from_slice(&[1, 2, 3, 4, 5]);
    vec2.extend_from_slice(&[10, 20, 30]);
    vec3.extend_from_slice(&[100, 200]);

    let after_data = alloc.memory_usage();

    let expected_content_bytes = 5 * size_of::<u8>() + 3 * size_of::<u32>() + 2 * size_of::<u64>();

    assert_eq!(after_data.content_bytes, expected_content_bytes);
    assert!(after_data.capacity_bytes >= expected_content_bytes);
    assert!(after_data.overhead_bytes > 0);
    assert!(after_data.memory_utilization_ratio() > 0.0);

    let _ = alloc.fresh_arena(3);
    let after_reset = alloc.memory_usage();

    assert_eq!(
        after_reset.content_bytes, 0,
        "Content bytes should be 0 after reset"
    );
    assert!(
        after_reset.capacity_bytes > 0,
        "Capacity bytes should be preserved"
    );
    assert!(
        after_reset.overhead_bytes > 0,
        "Overhead bytes should exist after reset"
    );
    let arena = alloc.fresh_arena(4);

    for _ in 0..10 {
        let vec = arena.alloc_vec::<u32>();
        for _ in 0..100 {
            vec.push(42);
        }
    }

    let alloc_usage = alloc.memory_usage();
    let expected_content = 10 * 100 * size_of::<u32>();
    assert_eq!(alloc_usage.content_bytes, expected_content);
    assert!(alloc_usage.capacity_bytes >= expected_content);
}

#[test]
fn test_chunk_consolidation() {
    let mut alloc = Arena::default();

    // First arena: allocate vectors to force multiple chunks
    let arena = alloc.fresh_arena(1);

    let mut original_addresses_u32 = Vec::new();
    let mut original_addresses_u64 = Vec::new();

    for _ in 0..100 {
        let vec_u32 = arena.alloc_vec::<u32>();
        let vec_u64 = arena.alloc_vec::<u64>();

        original_addresses_u32.push(vec_u32 as *mut _);
        original_addresses_u64.push(vec_u64 as *mut _);

        // Force capacity growth
        for i in 0..20 {
            vec_u32.push(i);
            vec_u64.push(i as u64);
        }
    }

    // First consolidation of the chunks
    let arena = alloc.fresh_arena(2);

    let mut first_addresses_u32 = Vec::new();
    let mut first_addresses_u64 = Vec::new();

    for _ in 0..100 {
        let vec_u32 = arena.alloc_vec::<u32>();
        let vec_u64 = arena.alloc_vec::<u64>();

        // Verify capacities are preserved
        assert!(vec_u32.capacity() >= 20);
        assert!(vec_u64.capacity() >= 20);

        first_addresses_u32.push(vec_u32 as *mut _);
        first_addresses_u64.push(vec_u64 as *mut _);
    }

    for i in 0..100 {
        assert_ne!(
            original_addresses_u32[i], first_addresses_u32[i],
            "u32 vector addresses should change after first consolidation"
        );
        assert_ne!(
            original_addresses_u64[i], first_addresses_u64[i],
            "u64 vector addresses should change after first consolidation"
        );
    }

    // Second consolidation
    let arena = alloc.fresh_arena(3);

    // Verify addresses remain stable after second consolidation
    for i in 0..100 {
        let vec_u32 = arena.alloc_vec::<u32>();
        let vec_u64 = arena.alloc_vec::<u64>();

        // Verify capacities are still preserved
        assert!(vec_u32.capacity() >= 20);
        assert!(vec_u64.capacity() >= 20);

        assert_eq!(
            vec_u32 as *mut _, first_addresses_u32[i],
            "u32 vector addresses should be stable after second consolidation"
        );
        assert_eq!(
            vec_u64 as *mut _, first_addresses_u64[i],
            "u64 vector addresses should be stable after second consolidation"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)] // Ignore Miri because this deliberately leaks memory
fn test_alloc_ignoring_drop() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);
    let _ = arena.alloc_val_ignore_drop(String::from("Hello, World!"));
    let vec = arena.alloc_vec_ignore_drop();
    vec.push(String::from("Hello, World 2!"));
    let _ = arena.alloc_slice_ignore_drop(&[String::from("Hello, World 3!")]);
}

#[test]
fn test_alloc_slice() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);
    let slice1 = arena.alloc_slice(&[1, 2, 3]);
    assert_eq!(slice1, &[1, 2, 3]);
    slice1[1] = 99;
    assert_eq!(slice1, &[1, 99, 3]);
    let slice2 = arena.alloc_slice(&[4, 5, 6, 7]);
    let val = arena.alloc_val(100);
    let slice3 = arena.alloc_slice::<u32>(&[]);
    let slice4 = arena.alloc_slice(&[1.0_f32, 2.0, 3.0]);
    let slice5 = arena.alloc_slice(&[4.0_f64, 5.0, 6.0]);
    assert_eq!(slice1, &[1, 99, 3]);
    assert_eq!(slice2, &[4, 5, 6, 7]);
    assert_eq!(*val, 100);
    assert_eq!(slice3, []);
    assert_eq!(slice4, &[1.0, 2.0, 3.0]);
    assert_eq!(slice5, &[4.0, 5.0, 6.0]);

    let usage = alloc.memory_usage();
    let expected_bytes = 8 * size_of::<u32>() + 3 * size_of::<f32>() + 3 * size_of::<u64>();
    assert_eq!(usage.content_bytes, expected_bytes);
}

#[test]
fn test_from_iter() {
    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);
    let a = arena.alloc_iter([(0, 0)].into_iter());
    let b = arena.alloc_iter([(1, 1), (2, 2), (3, 3)].into_iter());
    let c = arena.alloc_iter([(4, 4)].into_iter());

    assert_eq!(a, &[(0, 0)]);
    assert_eq!(b, &[(1, 1), (2, 2), (3, 3)]);
    assert_eq!(c, &[(4, 4)]);
}

#[test]
fn test_from_faulty_iter() {
    struct TooLongIterator(Vec<u32>);
    struct TooShortIterator(Vec<u32>);
    struct Filler(Vec<u32>);
    impl Iterator for TooLongIterator {
        type Item = u32;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop()
        }
    }
    impl ExactSizeIterator for TooLongIterator {
        fn len(&self) -> usize {
            1
        }
    }
    impl Iterator for TooShortIterator {
        type Item = u32;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop()
        }
    }
    impl ExactSizeIterator for TooShortIterator {
        fn len(&self) -> usize {
            100
        }
    }
    impl Iterator for Filler {
        type Item = u32;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop()
        }
    }
    impl ExactSizeIterator for Filler {
        fn len(&self) -> usize {
            self.0.len()
        }
    }

    let too_long = TooLongIterator(vec![3, 2, 1]);
    let too_short = TooShortIterator(vec![6, 5, 4]);
    let filler = Filler(vec![7; 97]);

    let mut alloc = Arena::default();
    let arena = alloc.fresh_arena(1);

    let a = arena.alloc_iter(too_long);
    assert_eq!(a, &[1]);

    let mut usage_before = MemoryUsage::default();
    arena.val_memory_usage(&mut usage_before);
    assert_eq!(usage_before.content_bytes, 1 * size_of::<u32>());

    let b = arena.alloc_iter(too_short);
    assert_eq!(b, &[4, 5, 6]);

    let mut usage_after = MemoryUsage::default();
    arena.val_memory_usage(&mut usage_after);
    assert_eq!(usage_after.content_bytes, 4 * size_of::<u32>());
    let reserved = 100 * size_of::<u32>();
    assert!(usage_before.capacity_bytes + reserved <= usage_after.capacity_bytes);

    let c = arena.alloc_iter(filler);
    assert_eq!(c.len(), 97);

    let mut usage_last = MemoryUsage::default();
    arena.val_memory_usage(&mut usage_last);
    assert!(usage_after.capacity_bytes == usage_last.capacity_bytes);
}

#[test]
fn test_memory_usage_invalidation() {
    let mut arena = ArenaGuard::new();

    let vec = arena.alloc_vec();
    vec.extend(&[42, 43, 44, 45, 46]);
    let slice = vec.as_mut_slice();
    unsafe { arena.memory_usage() };
    slice[2] = 99; // Miri: No UB
    //vec.push(100); // Miri: UB!
}

#[test]
fn test_alloc_str_slice() {
    let mut arena = ArenaGuard::new();
    let str = arena.alloc_str("Hello, world!");
    assert_eq!(str, "Hello, world!");
}

#[test]
fn test_alloc_string() {
    let mut arena = ArenaGuard::new();
    let string = arena.alloc_string("Hello");
    assert_eq!(string, "Hello");
    string.push_str(", world!");
    assert_eq!(string, "Hello, world!");
}

#[test]
fn test_sink() {
    let mut arena = ArenaGuard::new();
    let _ = arena.alloc_val(123);
    let mut sink = arena.alloc_sink();
    let a = sink.push(1);
    *a = 99;
    assert_eq!(&*sink, &[99]);
    sink[0] = 101;
    for i in 2..50 {
        sink.push(i);
    }

    let slice = sink.into_slice();
    let _ = arena.alloc_val(3); // Arena is usable again!
    let mut expected = Vec::new();
    for i in 1..50 {
        expected.push(i);
    }
    expected[0] = 101;
    assert_eq!(slice, &expected)
}

#[test]
fn test_string_vec_layout_comp() {
    assert_eq!(size_of::<String>(), size_of::<Vec<u8>>());
    assert_eq!(align_of::<String>(), align_of::<Vec<u8>>());
    let mut string = String::with_capacity(69);
    let mut vec = Vec::with_capacity(69);
    string.push_str("Hello, world!");
    vec.extend_from_slice(b"Hello, world!");
    let string_bytes: [usize; 3] = unsafe { transmute(string) };
    let vec_bytes: [usize; 3] = unsafe { transmute(vec) };
    let mut same_value = 0;
    same_value += (string_bytes[0] == vec_bytes[0]) as u8;
    same_value += (string_bytes[1] == vec_bytes[1]) as u8;
    same_value += (string_bytes[2] == vec_bytes[2]) as u8;
    assert_eq!(same_value, 2); // The len and capacity should match, the ptr shouldn't.
}

// Expect: error[E0499]: cannot borrow `alloc` as mutable more than once at a time
/// Test: getting memory usage invalidates the arena
/// ```compile_fail
/// let mut alloc = arena::Arena::default();
/// let arena = alloc.fresh_arena(1);
/// alloc.memory_usage();
/// let vec = arena.alloc_vec::<u32>();
/// vec.push(42);
/// ```
pub struct _CompileFailMemoryUsageInvalidatesArena1;

// Expect: error[E0499]: cannot borrow `alloc` as mutable more than once at a time
/// Test: getting memory usage invalidates the arena
/// ```compile_fail
/// let mut alloc = arena::Arena::default();
/// let arena = alloc.fresh_arena(0);
/// let vec = arena.alloc_vec::<u32>();
/// alloc.memory_usage();
/// vec.push(42);
/// ```
pub struct _CompileFailMemoryUsageInvalidatesArena2;

// Expect: error[E0597]: `alloc` does not live long enough
/// Test: allocated things can't outlive the allocator
/// ```compile_fail
/// let outlived_vec = {
///     let mut alloc = arena::Arena::default();
///     let mut arena = alloc.fresh_arena(1);
///
///     let vec = arena.alloc_vec::<u32>();
///     vec.push(42);
///     vec
/// };
/// ```
pub struct _CompileFailAllocationsCantOutliveArena;

// Expect: error[E0499]: cannot borrow `alloc` as mutable more than once at a time
/// Test: getting a new arena makes the old one unusable
/// ```compile_fail
/// let mut alloc = arena::Arena::default();
/// let mut arena = alloc.fresh_arena(1);
///
/// let vec = arena.alloc_vec::<u32>();
/// vec.push(42);
///
/// let mut arena = alloc.fresh_arena(2);
/// vec.push(43);
/// ```
pub struct _CompileFailNewArenaInvalidatesOldArena;

// Expect: evaluation of `arena::ArenaGuard::<'_>::check_drop_static::<std::string::String>::{constant#0}` failed
/// Test: attempt allocing a value with Drop implementation
/// ```compile_fail
/// let mut alloc = arena::Arena::default();
/// let mut arena = alloc.fresh_arena(1);
/// let _ = arena.alloc_val(String::from("Hello, World!"));
/// ```
pub struct _CompileFailForbidDropOnVal;

// Expect: evaluation of `arena::ArenaGuard::<'_>::check_drop_static::<std::string::String>::{constant#0}` failed
/// Test: attempt allocing a vec with Drop implementation
/// ```compile_fail
/// let mut alloc = arena::Arena::default();
/// let mut arena = alloc.fresh_arena(1);
/// let mut strings = arena.alloc_vec();
/// strings.push(String::from("Hello, World!"));
/// ```
pub struct _CompileFailForbidDropOnVec;

// Expect: error[E0080]: evaluation of `arena::ArenaGuard::<'_>::check_alignment_static::<main::_doctest_main_libs_arena_src_tests_rs_819_0::TestAlign32>::{constant#0}` failed
/// ```compile_fail
/// #[repr(align(32))]
/// struct TestAlign32;
/// let mut alloc = arena::Arena::default();
/// let arena = alloc.fresh_arena(1);
/// let _ = arena.alloc_val(TestAlign32);
/// ```
pub struct _CompileFailUnsupportedValAlign;

// Expect: error[E0080]: evaluation of `arena::ArenaGuard::<'_>::check_alignment_static::<main::_doctest_main_libs_arena_src_tests_rs_819_0::TestAlign32>::{constant#0}` failed
/// ```compile_fail
/// #[repr(align(32))]
/// struct TestAlign32;
/// let mut alloc = arena::Arena::default();
/// let arena = alloc.fresh_arena(1);
/// let _ = arena.alloc_vec::<TestAlign32>();
/// ```
pub struct _CompileFailUnsupportedVecAlign;
