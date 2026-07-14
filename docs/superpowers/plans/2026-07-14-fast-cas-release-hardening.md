# Fast CAS Release Hardening Implementation Plan

> **For agentic workers:** Execute inline with TDD and benchmark checkpoints.

**Goal:** Remove the runtime dependency, improve API/error ergonomics and concurrency documentation, add real contention tests and benchmarks, and use measurements to decide spin and inlining behavior before publishing 0.1.0.

**Architecture:** `CasCell` will directly own `AtomicU64` with the existing Acquire/Release/AcqRel contract. `FastCas` keeps the current public result and policy types while accepting `FnMut`; performance-only changes are retained only when Criterion measurements satisfy the approved threshold.

**Tech Stack:** Rust 2024, standard-library atomics, Criterion benchmarks, Cargo integration tests, rs-ci.

## Global Constraints

- Do not depend on `qubit-atomic` or add any other normal dependency.
- Criterion is allowed only as a development dependency.
- Preserve all public state values as `u64` and memory ordering as Acquire/Release/AcqRel.
- Keep tests under `tests/`; do not add inline test modules.
- Document ABA, retry liveness/fairness, stale observations, and repeated closure evaluation.
- Use `align-ci.sh` and the canonical rs-ci commands for formatting.
- Do not commit, push, rewrite history, or disturb existing local commits.
- Retain `spin_loop()` only if contention improves stably and uncontended performance regresses by no more than 5%.
- Retain `#[inline(always)]` only if isolated benchmark evidence justifies it.

---

### Task 1: Add the benchmark harness and capture the initial baseline

**Files:**
- Modify: `Cargo.toml`
- Create: `benches/fast_cas_bench.rs`

**Interfaces:**
- Produces Criterion groups for uncontended operations and 2/4/8-thread contention.
- Compares direct `AtomicU64`, `FastCas::once`, `FastCas::spin(16)`, and `FastCas::spin_yield(8, 64)`.

- [x] Add Criterion as a dev-dependency and register the `fast_cas_bench` harness.
- [x] Implement fixed-work contention benchmarks that retry outer operations after bounded conflicts.
- [x] Run `cargo bench --bench fast_cas_bench --no-run`.
- [x] Run the initial benchmark once to validate that every scenario produces measurements.

### Task 2: Replace `qubit-atomic` with `AtomicU64`

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `src/cas_cell.rs`

**Interfaces:**
- `CasCell::new/load/store/swap/compare_set/update/try_update` remain source-compatible.
- Atomic orderings remain Acquire, Release, AcqRel/Acquire.

- [x] Run a normal-dependency tree assertion and verify it fails because `qubit-atomic` is present.
- [x] Remove the normal dependency and replace the storage implementation with `AtomicU64`.
- [x] Run focused CasCell tests and the normal-dependency tree assertion.

### Task 3: Broaden operation closures to `FnMut`

**Files:**
- Modify: `tests/fast/fast_cas_tests.rs`
- Modify: `src/fast_cas.rs`

**Interfaces:**
- `FastCas::execute` accepts `F: FnMut(u64) -> FastCasDecision<R, E>`.
- `FastCas::update_by` accepts `F: FnMut(u64) -> Result<(u64, R), E>`.

- [x] Add tests whose closures mutate ordinary local counters.
- [x] Run the focused test and verify the existing `Fn` bounds reject it.
- [x] Change the bounds and mutable bindings minimally.
- [x] Re-run the focused tests and all FastCas tests.

### Task 4: Integrate `FastCasError` with the standard error ecosystem

**Files:**
- Modify: `tests/fast/fast_cas_error_tests.rs`
- Modify: `src/fast_cas_error.rs`

**Interfaces:**
- Implements `Display` when `E: Display`.
- Implements `Error` when `E: Error + 'static`.
- Abort exposes its business error through `source()`; Conflict has no source.

- [x] Add formatting, trait-bound, and source-chain tests.
- [x] Run the focused test and verify it fails before the implementations exist.
- [x] Add minimal `Display` and `Error` implementations.
- [x] Re-run the focused tests.

### Task 5: Strengthen concurrent behavior verification and documentation

**Files:**
- Modify: `tests/fast/fast_cas_tests.rs`
- Modify: `src/cas_cell.rs`
- Modify: `src/fast_cas.rs`
- Modify: `README.md`
- Modify: `README.zh_CN.md`

**Interfaces:**
- Adds a real multithreaded shared-state increment test.
- Documents ABA mitigation, unbounded retry liveness, lack of fairness, observation freshness, and closure replay.

- [x] Add a four-thread contention test with a fixed successful-update count.
- [x] Run it against the existing implementation.
- [x] Add concise English and Chinese concurrency-limit sections and matching rustdoc notes.
- [x] Run doctests and documentation with warnings denied.

### Task 6: Benchmark spin and inlining candidates independently

**Files:**
- Modify conditionally: `src/fast_cas.rs`

**Interfaces:**
- Immediate retry may call `std::hint::spin_loop()` before the next attempt.
- Hot generic methods use either `#[inline(always)]` or `#[inline]` according to isolated measurements.

- [x] Save a Criterion baseline for the completed non-performance changes.
- [x] Add `spin_loop()` only on non-yield retry paths and run the full benchmark comparison.
- [x] Retain or revert only that candidate according to the approved threshold.
- [x] Save a new baseline, change only `inline(always)` to `inline`, and compare again.
- [x] Retain the simpler inlining annotations unless `inline(always)` has a measured material benefit.
- [x] Run all behavior tests after the selected implementation is restored.

### Task 7: Canonical verification and release dry run

**Files:**
- Modify mechanically as needed by: `align-ci.sh`

**Interfaces:**
- Produces a clean rs-ci run and a verified crates.io package without publishing it.

- [x] Run `./align-ci.sh` using the project formatter configuration.
- [x] Run `./ci-check.sh` and confirm every stage passes.
- [x] Run `cargo publish --dry-run`.
- [x] Inspect `git status --short`, `git diff --check`, the package file list, and the final normal dependency tree.

## Benchmark decisions

- Retained `spin_loop()` before immediate retries. With 30 samples, a one-second warm-up, and a two-second measurement window, two-thread contention improved by 27.5% for `Spin(16)` and 37.4% for `SpinYield(8, 64)`; four-thread `Spin(16)` improved by 24.6%. Other contention cases showed no statistically significant regression, and uncontended changes stayed within 2%.
- Retained `#[inline(always)]` on `execute` and `update_by`. Replacing it with `#[inline]` regressed measured Spin contention across the tested thread counts, including an 18.3% two-thread regression for `Spin(16)`.
- Changed the one-shot `compare_update` helpers to ordinary `#[inline]`; their uncontended benchmark did not justify forced inlining.
