// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! An atomic `u64` cell with functional compare-and-swap updates.

use std::convert::Infallible;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

/// An atomic `u64` cell with reusable compare-and-swap update loops.
///
/// Update closures may run multiple times when concurrent writers cause CAS
/// conflicts. They should derive their result only from the supplied state and
/// avoid non-idempotent side effects.
///
/// Functional updates retry without fairness or a completion bound. Sustained
/// contention can therefore delay an update indefinitely. Comparisons use only
/// the current `u64` value and do not detect ABA changes; callers that must
/// detect an intermediate change should encode a generation counter in the
/// state word.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct CasCell {
    /// Atomic state word owned by this cell.
    state: AtomicU64,
}

impl CasCell {
    /// Creates a cell initialized with `value`.
    ///
    /// # Parameters
    ///
    /// - `value`: Initial state word.
    ///
    /// # Returns
    ///
    /// A new cell containing `value`.
    #[inline]
    pub fn new(value: u64) -> Self {
        Self {
            state: AtomicU64::new(value),
        }
    }

    /// Loads the current state word with acquire ordering.
    ///
    /// # Returns
    ///
    /// The currently stored value.
    #[inline]
    pub fn load(&self) -> u64 {
        self.state.load(Ordering::Acquire)
    }

    /// Stores a state word with release ordering.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to store.
    #[inline]
    pub fn store(&self, value: u64) {
        self.state.store(value, Ordering::Release);
    }

    /// Atomically replaces the state word and returns its previous value.
    ///
    /// # Parameters
    ///
    /// - `value`: Replacement value.
    ///
    /// # Returns
    ///
    /// The value stored before the swap.
    #[inline]
    pub fn swap(&self, value: u64) -> u64 {
        self.state.swap(value, Ordering::AcqRel)
    }

    /// Replaces `expected` with `next` when the current value matches.
    ///
    /// # Parameters
    ///
    /// - `expected`: Value required for the update to succeed.
    /// - `next`: Replacement value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the value was replaced.
    ///
    /// # Errors
    ///
    /// Returns the observed current value when it differs from `expected`.
    #[inline]
    pub fn compare_set(&self, expected: u64, next: u64) -> Result<(), u64> {
        self.state
            .compare_exchange(expected, next, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
    }

    /// Repeatedly computes and installs a new state until CAS succeeds.
    ///
    /// The operation receives each observed state and returns the replacement
    /// state together with a business output. It may be called more than once
    /// after conflicts. Only output from the committed attempt is returned.
    /// A panic from the operation propagates and leaves the state unchanged by
    /// that attempt.
    /// This unbounded loop does not guarantee fairness or completion under
    /// sustained contention.
    ///
    /// # Parameters
    ///
    /// - `operation`: State transition evaluated for each observed value.
    ///
    /// # Returns
    ///
    /// The business output from the successfully committed transition.
    #[inline]
    pub fn update<R, F>(&self, mut operation: F) -> R
    where
        F: FnMut(u64) -> (u64, R),
    {
        match self.try_update(|current| Ok::<(u64, R), Infallible>(operation(current))) {
            Ok(output) => output,
            Err(error) => match error {},
        }
    }

    /// Repeatedly computes and installs a fallible state transition.
    ///
    /// CAS conflicts retry without an attempt limit. A business error returned
    /// by `operation` stops immediately and leaves the state unchanged by that
    /// attempt. The operation may run more than once and should avoid
    /// non-idempotent side effects. A panic propagates and leaves the state
    /// unchanged by that attempt.
    /// This unbounded loop does not guarantee fairness or completion under
    /// sustained contention.
    ///
    /// # Parameters
    ///
    /// - `operation`: Fallible transition evaluated for each observed value.
    ///
    /// # Returns
    ///
    /// The business output from the successfully committed transition.
    ///
    /// # Errors
    ///
    /// Returns the business error produced by `operation` without retrying it.
    #[inline]
    pub fn try_update<R, E, F>(&self, mut operation: F) -> Result<R, E>
    where
        F: FnMut(u64) -> Result<(u64, R), E>,
    {
        let mut current = self.load();
        loop {
            let (next, output) = operation(current)?;
            match self.compare_set(current, next) {
                Ok(()) => return Ok(output),
                Err(actual) => current = actual,
            }
        }
    }
}
