# Current design

This document describes the current `qubit-fast-cas` API and behavior. The
[简体中文版本](design.zh_CN.md) covers the same contracts.

## Layers and data flow

`CasCell` owns one `AtomicU64`. Its primitive operations (`load`, `store`,
`swap`, and `compare_set`) expose a focused atomic state word. `update` and
`try_update` provide functional transitions: a closure computes a candidate
state and output, then a compare-and-set commits the candidate. A lost race
restarts the closure with the newly observed state. `try_update` also stops
when the closure returns a business error.

`FastCas` applies a bounded conflict policy (`Once`, `Spin`, or `SpinYield`)
to a `CasCell` and returns typed decisions, success values, or terminal errors
with attempt metadata. `FastCasState` is an alias for `CasCell`. State and
attempt values are `u64` and `usize`, respectively.

## Memory ordering

Atomic loads use Acquire ordering and stores use Release ordering. Successful
compare-and-set operations use AcqRel ordering; failed comparisons use Acquire
ordering. These orderings are part of the current implementation's behavior.

## Retries, ABA, and fairness

`CasCell::update` and `try_update` have no retry bound or fairness guarantee.
Under sustained contention an operation can be delayed indefinitely, and its
transition closure can run repeatedly. Such closures should be cheap and
should not perform non-idempotent side effects. `FastCas` bounds the number of
CAS attempts according to its selected policy; a conflict is reported when
that budget is exhausted.

For `CasCell` functional updates, a panic in the transition closure occurs
before compare-and-set, so that invocation has not committed its candidate
state. A successful compare-and-set is the commit point.

The state comparison observes only the current `u64` value. It cannot detect
an ABA sequence that changes a value and later restores it. Encode a
generation/version in the state word when intermediate transitions matter.
Success and error metadata describe the terminal attempt; another writer can
change shared state immediately afterward.

## One-shot output and commit order

`FastCas::compare_update_with` attempts one compare-and-set. It calls its
`FnOnce` output factory exactly once after a successful commit, passing the
expected and next values. On a conflict it returns without calling the
factory. If the factory panics, the state change has already committed and is
not rolled back. This ordering lets the factory consume captured values while
keeping it out of failed attempts.

## Downstream integration and migration boundary

As observed in the local checkout on 2026-11-12, `rs-cas` does not declare a
`qubit-fast-cas` dependency and its crate root does not re-export this crate;
the extraction design's proposed re-export is historical, not a current
compatibility guarantee. `rs-state-machine`, `rs-progress`, and `rs-executor`
declare direct `qubit-fast-cas` dependencies. Their inspected call sites use
the policy, state, executor, or higher-level state-machine APIs; no inspected
call site directly uses `compare_update_with`, so the `FnOnce` relaxation
requires no migration there. This is a dated local source observation, not a
claim about remote repository state.

The crate targets Rust 1.94+ and retains the `u64` state representation.
Compared with `qubit-cas` 0.8's `usize` state and `qubit_atomic::Atomic<usize>`
alias, callers may need to convert state values and replace other low-level
atomic operations with `CasCell` methods or a separately owned atomic. The
focused `load`, `store`, `swap`, and `compare_set` methods remain available.
