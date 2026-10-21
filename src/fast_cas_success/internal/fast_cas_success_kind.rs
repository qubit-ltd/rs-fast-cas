// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Decision path recorded by a successful fast CAS operation.

/// Identifies whether a successful operation updated the state or finished.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::fast_cas_success) enum FastCasSuccessKind {
    /// The operation successfully installed a state value.
    Updated,
    /// The operation completed without updating the state.
    Finished,
}
