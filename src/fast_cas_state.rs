// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Compatibility name for state used by [`crate::FastCas`].

use crate::CasCell;

/// Shared atomic state used by [`crate::FastCas`].
///
/// This alias keeps the established executor-oriented name while the concrete
/// state-owning primitive is exposed as [`CasCell`]. New code that only needs
/// unbounded functional updates should prefer [`CasCell`] directly.
///
/// This is not an alias for `qubit_atomic::Atomic<u64>`. Code migrating from
/// the former `Atomic<usize>` alias can keep using `load`, `store`, `swap`, and
/// `compare_set`; other low-level atomic operations should be expressed with
/// [`CasCell::update`] / [`CasCell::try_update`] or a separately owned atomic.
///
/// # Examples
///
/// ```
/// use qubit_fast_cas::FastCasState;
///
/// let state = FastCasState::new(1);
/// assert_eq!(state.load(), 1);
/// state.store(2);
/// assert_eq!(state.load(), 2);
/// ```
pub type FastCasState = CasCell;
