// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Lightweight compare-and-swap primitives for `u64` state words.
//!
//! [`CasCell`] owns an atomic state word and retries functional updates until
//! they commit or return a business error. [`FastCas`] adds bounded conflict
//! policies and attempt metadata for callers that need explicit control.
//!
//! Update closures may execute repeatedly after conflicts and should avoid
//! non-idempotent side effects.

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod cas_cell;
mod fast_cas;
mod fast_cas_decision;
mod fast_cas_error;
mod fast_cas_policy;
mod fast_cas_state;
mod fast_cas_success;

pub use cas_cell::CasCell;
pub use fast_cas::FastCas;
pub use fast_cas_decision::FastCasDecision;
pub use fast_cas_error::FastCasError;
pub use fast_cas_policy::FastCasPolicy;
pub use fast_cas_state::FastCasState;
pub use fast_cas_success::FastCasSuccess;
