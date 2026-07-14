// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::{
    Arc,
    Barrier,
};
use std::thread;

use qubit_fast_cas::CasCell;

/// Verifies the primitive atomic operations and the full `u64` range.
#[test]
fn test_cas_cell_primitive_operations_support_u64() {
    let cell = CasCell::new(u64::MAX - 1);

    assert_eq!(cell.load(), u64::MAX - 1);
    assert!(cell.compare_set(u64::MAX - 1, u64::MAX).is_ok());
    assert_eq!(cell.swap(7), u64::MAX);
    assert_eq!(cell.load(), 7);
    cell.store(9);
    assert_eq!(cell.load(), 9);
    assert_eq!(cell.compare_set(8, 10), Err(9));
}

/// Verifies that `update` returns output from the committed transition.
#[test]
fn test_cas_cell_update_commits_state_and_returns_output() {
    let cell = CasCell::new(4);

    let output = cell.update(|current| (current + 3, current * 2));

    assert_eq!(cell.load(), 7);
    assert_eq!(output, 8);
}

/// Verifies that business rejection leaves the state unchanged.
#[test]
fn test_cas_cell_try_update_rejects_without_mutation() {
    let cell = CasCell::new(5);

    let result =
        cell.try_update(|_current| Err::<(u64, ()), &'static str>("rejected"));

    assert_eq!(result, Err("rejected"));
    assert_eq!(cell.load(), 5);
}

/// Verifies that a lost CAS is retried against the newly observed state.
#[test]
fn test_cas_cell_update_retries_after_conflict() {
    let cell = Arc::new(CasCell::new(0));
    let before_store = Arc::new(Barrier::new(2));
    let after_store = Arc::new(Barrier::new(2));
    let worker_cell = Arc::clone(&cell);
    let worker_before_store = Arc::clone(&before_store);
    let worker_after_store = Arc::clone(&after_store);
    let worker = thread::spawn(move || {
        worker_before_store.wait();
        worker_cell.store(10);
        worker_after_store.wait();
    });
    let mut attempts = 0;

    let output = cell.update(|current| {
        attempts += 1;
        if attempts == 1 {
            before_store.wait();
            after_store.wait();
        }
        (current + 1, current)
    });

    worker.join().expect("conflict worker should finish");
    assert_eq!(attempts, 2);
    assert_eq!(output, 10);
    assert_eq!(cell.load(), 11);
}
