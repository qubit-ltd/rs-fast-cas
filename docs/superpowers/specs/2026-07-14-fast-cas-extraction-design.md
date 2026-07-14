# Fast CAS Extraction Design

## Goal

Create the standalone `qubit-fast-cas` crate in `rs-fast-cas`, move the
lightweight CAS executor out of `qubit-cas`, change its state word from
`usize` to `u64`, and add a reusable state-owning `CasCell` primitive.

## Architecture

`CasCell` is the lower-level primitive. It owns a standard-library `AtomicU64`
without external runtime dependencies and provides
load, store, swap, one-shot compare-set, and unbounded functional CAS updates.
Its update closures may be evaluated repeatedly after conflicts and must not
perform non-idempotent side effects.

`FastCas` is the policy layer. It operates on `CasCell`, adds bounded
`Once`/`Spin`/`SpinYield` conflict handling, and returns typed decision,
success, and error values with attempt metadata. `FastCasState` remains a type
alias for `CasCell` to ease migration from `qubit-cas`.

`qubit-cas` depends on `qubit-fast-cas`, removes its duplicated implementation,
and re-exports the fast CAS public API from both its crate root and its `fast`
module.

## Public types

- `CasCell`
- `FastCas`
- `FastCasDecision<R, E>`
- `FastCasError<E>`
- `FastCasPolicy`
- `FastCasSuccess<R>`
- `FastCasState = CasCell`

All state values exposed by these APIs are `u64`.

## CasCell behavior

- `new`, `load`, `store`, `swap`, and `compare_set` expose the atomic word.
- `update` retries conflicts without an attempt limit and returns output only
  from the successful update.
- `try_update` retries conflicts without an attempt limit but stops immediately
  when the closure returns a business error.
- Atomic memory ordering is implemented directly: Acquire loads, Release
  stores, and AcqRel successful compare-set operations with Acquire failures.

## Compatibility

Existing `qubit_cas::FastCas*` root imports remain valid through re-exports.
The state word changing from `usize` to `u64` is intentionally breaking for
callers whose domain values are `usize`. `FastCasState` also changes from an
alias for `qubit_atomic::Atomic<usize>` to an alias for `CasCell`: `load`,
`store`, `swap`, and `compare_set` remain available, but other low-level
`Atomic` methods are intentionally not forwarded. Callers should use
`CasCell::update`/`try_update` or explicitly own another atomic type when they
need operations outside the focused `CasCell` API.

## Testing

Integration tests cover primitive atomic operations, successful and rejected
functional updates, deterministic conflict retry, the full `u64` range, fast
CAS policies, decisions, success/error accessors, and compatibility re-exports
from `qubit-cas`.
