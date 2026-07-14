# Fast CAS Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `qubit-fast-cas` with `CasCell` and the `u64`-based `FastCas`, then make `qubit-cas` consume and re-export it.

**Architecture:** `CasCell` owns the atomic word and implements unbounded functional CAS updates. `FastCas` layers bounded retry policy and typed outcomes on top. `qubit-cas` contains no duplicate fast implementation.

**Tech Stack:** Rust 2024, `qubit-atomic`, Cargo integration tests.

## Global Constraints

- Preserve all pre-existing uncommitted changes in `rs-cas`.
- Do not commit, add, push, delete branches, or rewrite Git history.
- Keep all state words and state-returning APIs as `u64`.
- Put tests under `tests/`; do not add inline test modules.
- Document every function and method, including private helpers.
- Follow red-green-refactor for every behavior.

---

### Task 1: Scaffold the standalone crate and specify CasCell

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `LICENSE`, `README.md`, `README.zh_CN.md`
- Create: `src/lib.rs`, `src/cas_cell.rs`
- Create: `tests/lib_tests.rs`, `tests/cas_cell/mod.rs`,
  `tests/cas_cell/cas_cell_tests.rs`

**Interfaces:**
- Produces: `CasCell::new(u64)`, `load() -> u64`, `store(u64)`,
  `swap(u64) -> u64`, `compare_set(u64, u64) -> Result<(), u64>`,
  `update(F) -> R`, and `try_update(F) -> Result<R, E>`.

- [x] Write tests importing `qubit_fast_cas::CasCell` and asserting primitive operations, `u64::MAX` support, successful updates, business rejection without mutation, and retry after a deterministic CAS conflict.
- [x] Run the focused CasCell tests and verify failure because `CasCell` does not exist.
- [x] Implement the minimal documented `CasCell` API over `Atomic<u64>`.
- [x] Run the focused CasCell tests and verify all CasCell tests pass.

### Task 2: Move and convert FastCas

**Files:**
- Create: `src/fast_cas.rs`, `src/fast_cas_decision.rs`,
  `src/fast_cas_error.rs`, `src/fast_cas_policy.rs`,
  `src/fast_cas_state.rs`, `src/fast_cas_success.rs`
- Modify: `src/lib.rs`
- Create: matching `tests/fast/*_tests.rs` files and update
  `tests/lib_tests.rs`

**Interfaces:**
- Consumes: `CasCell::{load, compare_set}`.
- Produces: the existing FastCas API with every state value changed from
  `usize` to `u64`; `FastCasState` aliases `CasCell`.

- [x] Port the existing fast CAS integration tests, change state values and operation signatures to `u64`, and add a `u64::MAX - 1 -> u64::MAX` transition test.
- [x] Run the fast CAS tests and verify failure because the types are not exported.
- [x] Implement the six FastCas source modules using `CasCell` as storage.
- [x] Run `cargo test` and verify the full standalone crate passes.

### Task 3: Replace rs-cas implementation with dependency re-exports

**Files:**
- Modify: `../rs-cas/Cargo.toml`, `../rs-cas/src/lib.rs`
- Replace: `../rs-cas/src/fast/mod.rs` with re-exports
- Delete: duplicated `../rs-cas/src/fast/fast_cas*.rs` implementation files
- Modify: `../rs-cas/tests/fast/*.rs` state signatures from `usize` to `u64`

**Interfaces:**
- Consumes: `qubit-fast-cas = { version = "0.1", path = "../rs-fast-cas" }`.
- Produces: compatible root and `fast`-module re-exports from `qubit-cas`.

- [x] Update rs-cas tests first so they require `u64` fast CAS behavior and direct `fast`-module re-exports.
- [x] Run the focused rs-cas tests and verify they fail before dependency integration, or record dependency-resolution failure if the existing unpublished local dependency chain prevents compilation.
- [x] Add the dependency, replace the module implementation with re-exports, and remove `qubit-atomic` only if no remaining rs-cas production code uses it directly.
- [x] Run rs-cas tests; preserve and report any pre-existing external dependency-resolution blocker.

### Task 4: Documentation and verification

**Files:**
- Modify: `README.md`, `README.zh_CN.md`, and rs-cas README fast-CAS references where present.

**Interfaces:**
- Documents: installation, `CasCell`, `FastCas`, repeated-closure semantics,
  and migration from `usize` to `u64`.

- [x] Add concise English and Chinese usage examples for `CasCell` and `FastCas`.
- [x] Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo doc --no-deps` in `rs-fast-cas`.
- [x] Run the available rs-cas verification commands and inspect `git diff --check` and `git status --short` in both repositories.
