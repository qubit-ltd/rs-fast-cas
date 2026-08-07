// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::convert::Infallible;

use qubit_fast_cas::CasCell;

#[test]
fn test_cas_cell_default_initializes_to_zero() {
    let cell = CasCell::default();

    assert_eq!(cell.load(), 0);
}

#[test]
fn test_cas_cell_load_store_swap_and_compare_set() {
    let cell = CasCell::new(1);

    assert_eq!(cell.load(), 1);

    cell.store(2);
    assert_eq!(cell.load(), 2);

    assert_eq!(cell.swap(3), 2);
    assert_eq!(cell.load(), 3);

    assert_eq!(cell.compare_set(2, 4), Err(3));
    assert_eq!(cell.load(), 3);

    assert_eq!(cell.compare_set(3, 4), Ok(()));
    assert_eq!(cell.load(), 4);
}

#[test]
fn test_cas_cell_update_returns_committed_output() {
    let cell = CasCell::new(10);

    let output = cell.update(|current| (current + 5, current * 2));

    assert_eq!(output, 20);
    assert_eq!(cell.load(), 15);
}

#[test]
fn test_cas_cell_update_retries_after_conflict() {
    let cell = CasCell::new(0);
    let mut calls = 0;

    let output = cell.update(|current| {
        calls += 1;
        if current == 0 {
            cell.store(10);
            (1, "stale")
        } else {
            (current + 1, "committed")
        }
    });

    assert_eq!(calls, 2);
    assert_eq!(output, "committed");
    assert_eq!(cell.load(), 11);
}

#[test]
fn test_cas_cell_try_update_returns_business_error_without_state_change() {
    let cell = CasCell::new(7);

    let error = cell
        .try_update(|current| {
            assert_eq!(current, 7);
            Err::<(u64, ()), _>("reject")
        })
        .expect_err("business error should stop update");

    assert_eq!(error, "reject");
    assert_eq!(cell.load(), 7);
}

#[test]
fn test_cas_cell_try_update_retries_after_conflict() {
    let cell = CasCell::new(5);
    let mut calls = 0;

    let output = cell
        .try_update(|current| {
            calls += 1;
            if current == 5 {
                cell.store(20);
                Ok::<(u64, u64), Infallible>((6, current))
            } else {
                Ok((current + 2, current))
            }
        })
        .expect("retry should eventually commit");

    assert_eq!(calls, 2);
    assert_eq!(output, 20);
    assert_eq!(cell.load(), 22);
}
