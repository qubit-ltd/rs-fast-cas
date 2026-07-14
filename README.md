# Qubit Fast CAS

[![Rust CI](https://github.com/qubit-ltd/rs-fast-cas/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-fast-cas/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-fast-cas/coverage-badge.json)](https://qubit-ltd.github.io/rs-fast-cas/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-fast-cas.svg?color=blue)](https://crates.io/crates/qubit-fast-cas)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Lightweight compare-and-swap primitives for reusable `u64` state machines.

`CasCell` owns one atomic state word and provides unbounded functional update
loops. `FastCas` adds bounded spin/yield policies and typed success/error
metadata without allocation, hooks, or execution reports.

## Installation

```toml
[dependencies]
qubit-fast-cas = "0.1"
```

## CasCell

Use `CasCell` when conflicts are an internal concurrency detail and the update
should keep retrying until it commits or the operation returns a business
error.

```rust
use qubit_fast_cas::CasCell;

let state = CasCell::new(10);
let previous = state.update(|current| (current + 1, current));

assert_eq!(previous, 10);
assert_eq!(state.load(), 11);
```

`update` and `try_update` closures may run more than once after concurrent CAS
conflicts. Keep them cheap and avoid non-idempotent side effects.

## FastCas

Use `FastCas` when callers need an explicit conflict budget and attempt
metadata.

```rust
use qubit_fast_cas::{FastCas, FastCasDecision, FastCasState};

let state = FastCasState::new(0);
let success = FastCas::spin_yield(8, 64)
    .execute(&state, |current| {
        if current == 0 {
            FastCasDecision::update(1, "started")
        } else {
            FastCasDecision::abort("already started")
        }
    })
    .expect("state should transition");

assert_eq!(success.current(), 1);
assert_eq!(success.into_output(), "started");
```

`FastCasState` is an alias for `CasCell`. All state values use `u64`.

## Migrating from qubit-cas 0.8

Fast CAS state values changed from `usize` to `u64`. `FastCasState` also changed
from an alias for `qubit_atomic::Atomic<usize>` to an alias for `CasCell`.
Primitive `load`, `store`, `swap`, and `compare_set` operations remain
available. Replace other `Atomic` operations with `CasCell::update` or
`try_update`, or own a separate atomic type when lower-level operations are
required.

## License

Licensed under the Apache License, Version 2.0.
