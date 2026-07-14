# Qubit Fast CAS

面向可复用 `u64` 状态机的轻量级 compare-and-swap 原语。

`CasCell` 持有一个原子状态字并提供无界重试的函数式更新循环；`FastCas`
在此基础上增加有界 spin/yield 策略以及强类型的成功、错误和尝试次数信息，
不引入分配、hooks 或执行报告。

## 安装

```toml
[dependencies]
qubit-fast-cas = "0.1"
```

## CasCell

当 CAS 冲突只是内部并发细节，并且更新应持续重试直至提交或业务逻辑拒绝时，
使用 `CasCell`。

```rust
use qubit_fast_cas::CasCell;

let state = CasCell::new(10);
let previous = state.update(|current| (current + 1, current));

assert_eq!(previous, 10);
assert_eq!(state.load(), 11);
```

并发冲突发生后，`update` 和 `try_update` 的闭包可能执行多次。因此闭包应保持
低开销，并避免不可重复执行的副作用。

## FastCas

当调用方需要明确的冲突尝试预算和尝试次数信息时，使用 `FastCas`。

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

`FastCasState` 是 `CasCell` 的类型别名。所有状态值均使用 `u64`。

## 从 qubit-cas 0.8 迁移

Fast CAS 状态值从 `usize` 改为 `u64`。`FastCasState` 也从
`qubit_atomic::Atomic<usize>` 的别名改为 `CasCell` 的别名。`load`、`store`、
`swap` 和 `compare_set` 仍可直接使用；其他 `Atomic` 操作应改用
`CasCell::update`/`try_update`，或在确实需要底层操作时自行持有独立的原子类型。

## 许可证

本项目使用 Apache License 2.0。
