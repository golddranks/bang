use super::*;
use std::ptr::addr_eq;

#[test]
fn test_no_allocation() {
    let mut _arena = Arena::new();
}

#[test]
fn test_empty_vec_allocation() {
    let mut arena = Arena::new();
    let _vec = arena.allocate_vec::<u32>();
}

#[test]
fn test_basic_vec_allocation() {
    let mut arena = Arena::new();

    let vec = arena.allocate_vec::<u32>();
    assert!(vec.is_empty());

    vec.push(42);
    vec.push(100);

    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], 42);
    assert_eq!(vec[1], 100);
}

#[test]
fn test_basic_val_allocation() {
    let mut arena = Arena::new();

    let val1 = arena.allocate_val(42u32);
    let val2 = arena.allocate_val(100u32);

    assert_eq!(*val1, 42);
    assert_eq!(*val2, 100);

    *val1 = 99;
    assert_eq!(*val1, 99);
    assert_eq!(*val2, 100);
}

#[test]
fn test_same_vec_layout() {
    let mut arena = Arena::new();

    let vec1 = arena.allocate_vec::<u8>();
    let vec2 = arena.allocate_vec::<u8>();

    vec1.extend_from_slice(&[1, 2, 3]);
    vec2.extend_from_slice(&[4, 5, 6]);

    assert_eq!(vec1, &[1, 2, 3]);
    assert_eq!(vec2, &[4, 5, 6]);
}

#[test]
fn test_multiple_vec_allocations() {
    let mut arena = Arena::new();

    let vec1 = arena.allocate_vec::<u8>();
    let vec2 = arena.allocate_vec::<u16>();
    let vec3 = arena.allocate_vec::<u32>();
    let vec4 = arena.allocate_vec::<u64>();
    let vec5 = arena.allocate_vec::<u8>();

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
    let mut arena = Arena::new();

    let vec1 = arena.allocate_vec::<u8>();
    let vec2 = arena.allocate_vec::<u16>();
    let vec4 = arena.allocate_vec::<u32>();
    let vec8 = arena.allocate_vec::<u64>();
    let vec16 = arena.allocate_vec::<u128>();
    let vec8_8 = arena.allocate_vec::<[u64; 2]>();

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
    let mut arena = Arena::new();

    let val1 = arena.allocate_val(0x42_u8);
    let val2 = arena.allocate_val(0x4243_u16);
    let val3 = arena.allocate_val(0x42434445_u32);
    let val4 = arena.allocate_val(0x4243444546474849_u64);
    let val5 = arena.allocate_val(0x42434445464748495051525354555657_u128);

    assert_eq!(*val1, 0x42u8);
    assert_eq!(*val2, 0x4243u16);
    assert_eq!(*val3, 0x42434445u32);
    assert_eq!(*val4, 0x4243444546474849u64);
    assert_eq!(*val5, 0x42434445464748495051525354555657_u128);

    let usage = arena.memory_usage();
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

    let usage = arena.memory_usage();
    assert_eq!(usage.content_bytes, expected_content);
}

#[test]
fn test_vec_reset() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(1);

    let vec = arena.allocate_vec::<u32>();
    vec.push(42);
    assert_eq!(vec.len(), 1);

    let arena = alloc.new_arena(2);
    let vec = arena.allocate_vec::<u32>();
    assert!(vec.is_empty());
}

#[test]
fn test_val_reset() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(1);

    let val1 = arena.allocate_val(42u32);
    let val2 = arena.allocate_val(43u64);
    assert_eq!(*val1, 42u32);
    assert_eq!(*val2, 43u64);

    let arena = alloc.new_arena(2);

    let new_val1 = arena.allocate_val(100u32);
    let new_val2 = arena.allocate_val(200u64);
    assert_eq!(*new_val1, 100u32);
    assert_eq!(*new_val2, 200u64);
}

#[test]
fn test_large_vec_allocation() {
    let mut arena = Arena::new();

    let vec = arena.allocate_vec::<u32>();
    for i in 0..1000 {
        vec.push(i);
    }

    for i in 0..1000 {
        assert_eq!(vec[i], i as u32);
    }

    let usage = arena.memory_usage();
    let expected_content = 1000 * size_of::<u32>();
    assert_eq!(usage.content_bytes, expected_content);
}

#[test]
fn test_large_val_allocation() {
    let mut arena = Arena::new();

    let large_value = arena.allocate_val([42u64; 32]);

    assert_eq!(large_value[0], 42);
    assert_eq!(large_value[31], 42);

    large_value[10] = 100;
    large_value[20] = 200;

    assert_eq!(large_value[10], 100);
    assert_eq!(large_value[20], 200);

    let large_value2 = arena.allocate_val([84u64; 32]);

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

    let usage = arena.memory_usage();
    let expected_content = 2 * 32 * size_of::<u64>();
    assert_eq!(usage.content_bytes, expected_content);
}

#[test]
fn test_mixed_vec_and_val_allocation() {
    let mut arena = Arena::new();

    let vec1 = arena.allocate_vec::<u32>();
    let val1 = arena.allocate_val(42u64);
    let vec2 = arena.allocate_vec::<u8>();
    let val2 = arena.allocate_val(43u16);

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

    let usage = arena.memory_usage();
    let expected_content =
        size_of::<u64>() + size_of::<u16>() + (2 * size_of::<u32>()) + (3 * size_of::<u8>());
    assert_eq!(usage.content_bytes, expected_content);

    *val1 = 99;
    *val2 = 999;

    assert_eq!(*val1, 99u64);
    assert_eq!(*val2, 999u16);
}

#[test]
fn test_zero_sized_val() {
    let mut arena = Arena::new();

    let empty1 = arena.allocate_val(());
    let empty2 = arena.allocate_val(());

    assert!(addr_eq(empty1, empty2));
}

#[test]
fn test_zero_sized_vec() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(0);

    let vec = arena.allocate_vec::<()>();

    assert_eq!(vec.capacity(), usize::MAX);
    assert_eq!(vec.len(), 0);

    vec.push(());

    assert_eq!(vec.capacity(), usize::MAX);
    assert_eq!(vec.len(), 1);
}

#[test]
fn test_val_memory_usage() {
    let mut arena = Arena::new();

    // Check initial memory usage
    let empty_usage = arena.memory_usage();
    assert_eq!(empty_usage.content_bytes, 0);
    assert_eq!(empty_usage.capacity_bytes, 80);

    // Allocate small values
    let _val1 = arena.allocate_val(1u8);
    let _val2 = arena.allocate_val(2u16);
    let _val3 = arena.allocate_val(3u32);

    // Check memory usage after small allocations
    let small_alloc_usage = arena.memory_usage();
    assert!(small_alloc_usage.capacity_bytes > 0);

    // Calculate exact content bytes for small allocations
    let expected_small_content = size_of::<u8>() + size_of::<u16>() + size_of::<u32>();
    assert_eq!(small_alloc_usage.content_bytes, expected_small_content);
    assert!(small_alloc_usage.total_bytes() == empty_usage.total_bytes());

    // Allocate a large array
    let _large_val = arena.allocate_val([42u64; 64]); // 512 bytes

    // Check memory usage after large allocation
    let large_alloc_usage = arena.memory_usage();
    assert!(large_alloc_usage.capacity_bytes > small_alloc_usage.capacity_bytes);

    // Calculate exact content bytes after large allocation
    let expected_large_content = expected_small_content + 64 * size_of::<u64>();
    assert_eq!(large_alloc_usage.content_bytes, expected_large_content);
    assert!(large_alloc_usage.total_bytes() > small_alloc_usage.total_bytes());

    // Create a new arena with allocator to test reset
    let mut alloc = ArenaContainer::default();
    let arena1 = alloc.new_arena(0);

    // Make some allocations
    let _v1 = arena1.allocate_val(100u32);
    let _v2 = arena1.allocate_val(200u64);

    let usage1 = alloc.memory_usage();
    let expected_content1 = size_of::<u32>() + size_of::<u64>();
    assert_eq!(usage1.content_bytes, expected_content1);

    // Get a new arena (which resets the allocations)
    let arena2 = alloc.new_arena(1);

    // Allocate again
    let _v3 = arena2.allocate_val(300u32);

    let usage2 = alloc.memory_usage();
    let expected_content2 = size_of::<u32>();
    assert_eq!(usage2.content_bytes, expected_content2);

    // Total memory should be the same after reset and reallocation
    // (since we're reusing memory)
    assert!(usage2.total_bytes() == usage1.total_bytes());
}

#[test]
fn test_multiple_vectors_same_type() {
    let mut arena = Arena::new();

    let vectors: Vec<_> = (0..10)
        .map(|i| {
            let vec = arena.allocate_vec::<u32>();
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
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(0);

    // Grow u32 vecs (4-byte alignment)
    let vec_u32_a = arena.allocate_vec::<u32>();
    let vec_u32_b = arena.allocate_vec::<u32>();
    for i in 0..1000 {
        vec_u32_a.push(i);
    }
    for i in 0..500 {
        vec_u32_b.push(i);
    }

    let arena = alloc.new_arena(1);
    let vec_u32 = arena.allocate_vec::<u32>();
    let vec_f32 = arena.allocate_vec::<f32>(); // f32 also has 4-byte alignment
    assert!(
        vec_u32.capacity() >= 1000,
        "u32 vector capacity should be reused"
    );
    assert!(
        vec_f32.capacity() >= 500,
        "Vectors with same alignment should share capacity pool"
    );
}

#[repr(align(32))]
struct Test;

#[test]
#[should_panic(expected = "Unsupported alignment")]
fn test_unsupported_alignment_vec() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(0);

    let _ = arena.allocate_vec::<Test>();
}

#[test]
#[should_panic(expected = "Unsupported alignment")]
fn test_unsupported_alignment_val() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(0);

    let _ = arena.allocate_val(Test);
}

#[test]
fn test_vec_memory_usage() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(0);

    let initial_usage = arena.memory_usage();
    assert_eq!(initial_usage.content_bytes, 0);
    assert_eq!(initial_usage.capacity_bytes, 5 * 4 * 4);

    let vec1 = arena.allocate_vec::<u8>();
    vec1.extend_from_slice(&[1, 2, 3, 4, 5]);
    let _vec1_2 = arena.allocate_vec::<u8>();
    let _vec2 = arena.allocate_vec::<u32>();
    let _vec3 = arena.allocate_vec::<u64>();
    let _vec3_2 = arena.allocate_vec::<(u64, u64)>();

    let after_alloc = arena.memory_usage();
    assert!(after_alloc.capacity_bytes > 0);
    // Content bytes should be exactly 5 bytes (for vec1)
    assert_eq!(after_alloc.content_bytes, 5);

    let arena = alloc.new_arena(1);
    let vec1 = arena.allocate_vec::<u8>();
    let vec2 = arena.allocate_vec::<u32>();
    let vec3 = arena.allocate_vec::<u64>();

    vec1.extend_from_slice(&[1, 2, 3, 4, 5]);
    vec2.extend_from_slice(&[10, 20, 30]);
    vec3.extend_from_slice(&[100, 200]);

    let after_data = arena.memory_usage();

    let expected_content_bytes = 5 * size_of::<u8>() + 3 * size_of::<u32>() + 2 * size_of::<u64>();

    assert_eq!(after_data.content_bytes, expected_content_bytes);
    assert!(after_data.capacity_bytes >= expected_content_bytes);
    assert!(after_data.overhead_bytes > 0);
    assert!(after_data.memory_utilization_ratio() > 0.0);

    let arena = alloc.new_arena(2);
    let after_reset = arena.memory_usage();

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

    for _ in 0..10 {
        let vec = arena.allocate_vec::<u32>();
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
    let mut alloc = ArenaContainer::default();

    // First arena: allocate vectors to force multiple chunks
    let arena = alloc.new_arena(0);

    let mut original_addresses_u32 = Vec::new();
    let mut original_addresses_u64 = Vec::new();

    for _ in 0..100 {
        let vec_u32 = arena.allocate_vec::<u32>();
        let vec_u64 = arena.allocate_vec::<u64>();

        original_addresses_u32.push(vec_u32 as *mut _);
        original_addresses_u64.push(vec_u64 as *mut _);

        // Force capacity growth
        for i in 0..20 {
            vec_u32.push(i);
            vec_u64.push(i as u64);
        }
    }

    // First consolidation of the chunks
    let arena = alloc.new_arena(1);

    let mut first_addresses_u32 = Vec::new();
    let mut first_addresses_u64 = Vec::new();

    for _ in 0..100 {
        let vec_u32 = arena.allocate_vec::<u32>();
        let vec_u64 = arena.allocate_vec::<u64>();

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
    let arena = alloc.new_arena(2);

    // Verify addresses remain stable after second consolidation
    for i in 0..100 {
        let vec_u32 = arena.allocate_vec::<u32>();
        let vec_u64 = arena.allocate_vec::<u64>();

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
#[cfg_attr(miri, ignore)]
fn test_allocate_ignoring_drop() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(1);
    let _ = arena.allocate_val_ignore_drop(String::from("Hello, World!"));
    let vec = arena.allocate_vec_ignore_drop();
    vec.push(String::from("Hello, World 2!"));
    let _ = arena.allocate_slice_ignore_drop(&[String::from("Hello, World 3!")]);
}

#[test]
fn test_allocate_slice() {
    let mut alloc = ArenaContainer::default();
    let arena = alloc.new_arena(1);
    let slice1 = arena.allocate_slice(&[1, 2, 3]);
    assert_eq!(slice1, &[1, 2, 3]);
    slice1[1] = 99;
    assert_eq!(slice1, &[1, 99, 3]);
    let slice2 = arena.allocate_slice(&[4, 5, 6, 7]);
    let val = arena.allocate_val(100);
    let slice3 = arena.allocate_slice::<u32>(&[]);
    let slice4 = arena.allocate_slice(&[1.0_f32, 2.0, 3.0]);
    let slice5 = arena.allocate_slice(&[4.0_f64, 5.0, 6.0]);
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

/// Test: getting memory usage invalidates the arena
/// ```compile_fail
/// use arena::ArenaContainer;
/// let mut alloc = ArenaContainer::default();
/// let arena = alloc.new_arena(1);
/// alloc.memory_usage();
/// let vec = arena.allocate_vec::<u32>();
/// vec.push(42);
/// ```
pub struct _CompileFailMemoryUsageInvalidatesArena1;

/// Test: getting memory usage invalidates the arena
/// ```compile_fail
/// use arena::ArenaContainer;
/// let mut alloc = ArenaContainer::default();
/// let arena = alloc.new_arena(0);
/// let vec = arena.allocate_vec::<u32>();
/// alloc.memory_usage();
/// vec.push(42);
/// ```
pub struct _CompileFailMemoryUsageInvalidatesArena2;

/// Test: allocated things can't outlive the allocator
/// ```compile_fail
/// use arena::ArenaContainer;
/// let outlived_vec = {
///     let mut alloc = ArenaContainer::default();
///     let mut arena = alloc.new_arena(1);
///
///     let vec = arena.allocate_vec::<u32>();
///     vec.push(42);
///     vec
/// };
/// ```
pub struct _CompileFailAllocationsCantOutliveArena;

/// Test: getting a new arena makes the old one unusable
/// ```compile_fail
/// use arena::ArenaContainer;
/// let mut alloc = ArenaContainer::default();
/// let mut arena = alloc.new_arena(1);
///
/// let vec = arena.allocate_vec::<u32>();
/// vec.push(42);
///
/// let mut arena = alloc.new_arena(2);
/// vec.push(43);
/// ```
pub struct _CompileFailNewArenaInvalidatesOldArena;

/// Test: attempt allocing a value with Drop implementation
/// ```compile_fail
/// use arena::ArenaContainer;
/// let mut alloc = ArenaContainer::default();
/// let mut arena = alloc.new_arena(1);
/// let _ = arena.allocate_val(String::from("Hello, World!"));
/// ```
pub struct _CompileFailForbidDropOnVal;

/// Test: attempt allocing a vec with Drop implementation
/// ```compile_fail
/// use arena::ArenaContainer;
/// let mut alloc = ArenaContainer::default();
/// let mut arena = alloc.new_arena(1);
/// let mut strings = arena.allocate_vec();
/// strings.push(String::from("Hello, World!"));
/// ```
pub struct _CompileFailForbidDropOnVec;
