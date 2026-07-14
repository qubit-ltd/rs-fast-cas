// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::io;

use qubit_fast_cas::{
    FastCas,
    FastCasDecision,
    FastCasError,
    FastCasState,
};

type TestDecision = FastCasDecision<u64, &'static str>;
type TestOperation = fn(u64) -> TestDecision;

fn abort_stop(_current: u64) -> TestDecision {
    FastCasDecision::abort("stop")
}

/// Requires `T` to implement the standard error trait.
fn assert_std_error<T: Error>() {}

#[test]
fn test_fast_cas_error_accessors_for_abort() {
    let state = FastCasState::new(4);
    let abort_stop: TestOperation = abort_stop;
    let error = FastCas::once()
        .execute(&state, abort_stop)
        .expect_err("operation should abort");

    assert_eq!(error.current(), 4);
    assert_eq!(error.attempts(), 1);
    assert!(error.is_abort());
    assert!(!error.is_conflict());
    assert_eq!(error.error(), Some(&"stop"));
    assert_eq!(error.into_error(), Some("stop"));
}

#[test]
fn test_fast_cas_error_accessors_for_conflict() {
    let error = FastCasError::<&'static str>::Conflict {
        current: 9,
        attempts: 2,
    };

    assert_eq!(error.current(), 9);
    assert_eq!(error.attempts(), 2);
    assert!(!error.is_abort());
    assert!(error.is_conflict());
    assert_eq!(error.error(), None);
    assert_eq!(error.into_error(), None);
}

/// Verifies abort failures format their context and expose the business cause.
#[test]
fn test_fast_cas_error_abort_implements_error() {
    assert_std_error::<FastCasError<io::Error>>();
    let error = FastCasError::Abort {
        current: 4,
        error: io::Error::other("stop"),
        attempts: 2,
    };

    assert_eq!(
        error.to_string(),
        "fast CAS operation aborted after 2 attempts at state 4: stop"
    );
    let source = error.source().expect("abort should expose its cause");
    assert_eq!(source.to_string(), "stop");
}

/// Verifies conflict failures format their context without a source error.
#[test]
fn test_fast_cas_error_conflict_implements_error() {
    assert_std_error::<FastCasError<io::Error>>();
    let error = FastCasError::<io::Error>::Conflict {
        current: 9,
        attempts: 3,
    };

    assert_eq!(
        error.to_string(),
        "fast CAS operation exhausted its retry policy after 3 attempts; latest state is 9"
    );
    assert!(error.source().is_none());
}
