//! Native storage boundary, copy-on-write and recoverable-failure tests.

use super::*;
use std::cell::Cell;

thread_local! {
    pub(super) static FAIL_ALLOCATION: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn fail_allocation() -> bool {
    FAIL_ALLOCATION.with(|fail| fail.replace(false))
}

#[test]
fn copies_ranges_and_overlapping_sources_have_value_semantics() {
    let original = SharedBytes::from_slice(&[1, 2, 3, 4]).unwrap();
    let mut copied = original.clone();
    copied.set_at(0, 9).unwrap();
    assert_eq!(original.as_ref(), &[1, 2, 3, 4]);
    assert_eq!(copied.as_ref(), &[9, 2, 3, 4]);
    let source = copied.clone();
    copied.copy_from(&source, 0, 1, 3).unwrap();
    assert_eq!(copied.as_ref(), &[9, 9, 2, 3]);
    copied.concat(&copied.clone()).unwrap();
    assert_eq!(copied.as_ref(), &[9, 9, 2, 3, 9, 9, 2, 3]);
}

#[test]
fn proper_and_empty_slices_release_parent_capacity() {
    let mut original = SharedBytes::from_slice(&[1, 2, 3, 4]).unwrap();
    original.reserve(1024).unwrap();
    let full = original.slice(0, 4).unwrap();
    assert!(Arc::ptr_eq(&full.0, &original.0));
    let part = original.slice(1, 2).unwrap();
    assert!(!Arc::ptr_eq(&part.0, &original.0));
    assert_eq!(part.capacity(), 2);
    assert_eq!(original.slice(4, 0).unwrap().capacity(), 0);
    assert_eq!(
        original.slice(usize::MAX, 1),
        Err(ByteStorageError::InvalidRange)
    );
}

#[test]
fn native_empty_results_release_unused_read_capacity() {
    let empty = SharedBytes::from_vec(Vec::with_capacity(1024 * 1024)).unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.capacity(), 0);
}

#[test]
fn last_owner_releases_backing_while_small_slices_remain_live() {
    let original = SharedBytes::zeroed(262144).unwrap();
    let backing = Arc::downgrade(&original.0);
    let copy = original.clone();
    let tiny = original.slice(0, 1).unwrap();
    drop(original);
    assert!(backing.upgrade().is_some());
    drop(copy);
    assert!(backing.upgrade().is_none());
    assert_eq!(tiny.as_ref(), &[0]);
}

#[test]
fn shrinking_retains_capacity_but_newly_exposed_bytes_are_zero() {
    let mut bytes = SharedBytes::from_slice(&[1, 2, 3]).unwrap();
    bytes.resize(1).unwrap();
    assert_eq!(bytes.capacity(), 3);
    bytes.resize(3).unwrap();
    assert_eq!(bytes.as_ref(), &[1, 0, 0]);
    bytes.resize(4).unwrap();
    assert_eq!(bytes.capacity(), 64);
    bytes.resize(0).unwrap();
    assert_eq!(bytes.capacity(), 64);
    bytes.trim_capacity().unwrap();
    assert_eq!(bytes.capacity(), 0);
}

#[test]
fn allocation_and_limit_failures_preserve_data_capacity_and_sharing() {
    let original = SharedBytes::from_slice(&[1, 2, 3]).unwrap();
    let mut copy = original.clone();
    let identity = Arc::as_ptr(&copy.0);
    FAIL_ALLOCATION.with(|fail| fail.set(true));
    assert_eq!(
        copy.set_at(0, 9),
        Err(ByteStorageError::AllocationFailed(3))
    );
    assert_eq!(Arc::as_ptr(&copy.0), identity);
    assert_eq!(copy.capacity(), 3);
    assert_eq!(copy, original);
    assert_eq!(
        copy.resize(MAX_BYTE_BUFFER_LENGTH + 1),
        Err(ByteStorageError::LimitExceeded)
    );
    assert_eq!(copy.fill(3, 1, 9), Err(ByteStorageError::InvalidRange));
    assert_eq!(copy, original);
}

#[test]
fn equality_ignores_capacity_and_display_is_bounded() {
    let original = SharedBytes::from_slice(&[0, 255]).unwrap();
    let mut reserved = original.clone();
    reserved.reserve(128).unwrap();
    assert_eq!(original, reserved);
    assert_eq!(
        original.to_string(),
        "ByteBuffer(Length := 2, Hex := '00FF')"
    );
    let long = SharedBytes::from_slice(&[255; 64]).unwrap();
    assert!(long.to_string().ends_with("...')"));
    assert_eq!(long.to_string().matches("FF").count(), 32);
}
