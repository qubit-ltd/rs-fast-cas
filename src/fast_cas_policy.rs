// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Retry policy for fast compare-and-swap operations.

#[path = "fast_cas_policy/fast_cas_policy.rs"]
mod policy;

pub use policy::FastCasPolicy;
