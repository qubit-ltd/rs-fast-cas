# 当前设计

本文说明 `qubit-fast-cas` 当前 API 与行为约定。[English version](design.md)
覆盖相同的设计主题。

## 分层与数据流

`CasCell` 持有一个 `AtomicU64`。`load`、`store`、`swap` 和 `compare_set`
提供聚焦的原子状态字操作。`update` 与 `try_update` 则接受函数式状态转换：闭包
计算候选状态和输出，再由 compare-and-set 尝试提交。若其他线程先完成写入，闭包会
使用新观测到的状态重新计算。`try_update` 还会在闭包返回业务错误时立即结束。

`FastCas` 在 `CasCell` 上应用有界冲突策略（`Once`、`Spin` 或 `SpinYield`），并
返回带尝试次数信息的强类型决策、成功值或终止错误。`FastCasState` 是 `CasCell`
的类型别名。状态值使用 `u64`，尝试次数使用 `u32`。

## 内存序

原子读取使用 Acquire，写入使用 Release。compare-and-set 成功时使用 AcqRel，
比较失败时使用 Acquire。这些内存序属于当前实现的行为约定。

## 重试、ABA 与公平性

`CasCell::update` 和 `try_update` 没有重试上限，也不保证公平性。持续竞争时，某次
操作可能长期得不到执行；状态转换闭包也可能因竞争而重复运行。因此闭包应保持轻量，
避免产生不可重复执行的副作用。`FastCas` 根据所选策略限制 CAS 尝试次数；耗尽预算
后会报告冲突。

对于 `CasCell` 函数式更新，转换闭包若发生 panic，compare-and-set 尚未执行，本次候选
状态也就尚未提交。成功的 compare-and-set 才是提交点。

比较只检查当前 `u64` 值，无法识别先改变、随后又恢复原值的 ABA 序列。如果业务必须
识别中间状态，应在状态字中编码版本号。成功或错误元数据描述终止尝试时的观测；其他
线程可能紧接着再次修改共享状态。

## 一次性输出与提交顺序

`FastCas::compare_update_with` 只尝试一次 compare-and-set。提交成功后，它恰好调用
一次 `FnOnce` 输出工厂，并传入 expected 与 next 值。冲突时不会调用工厂。如果工厂
发生 panic，状态变更已经提交，不会回滚。这样工厂可以消费捕获值，同时不会在失败
尝试中运行。

## 下游集成与迁移边界

截至 2026-11-12 对本地 checkout 的检查，`rs-cas` 未声明 `qubit-fast-cas` 依赖，
crate 根模块也没有重导出本 crate；提取设计中计划的重导出属于历史方案，不是当前的
兼容性保证。`rs-state-machine`、`rs-progress` 与 `rs-executor` 声明了直接依赖。已检查
的调用点使用策略、状态、执行器或更高层状态机 API；没有调用点直接使用
`compare_update_with`，因此将其约束放宽为 `FnOnce` 无需下游迁移。这是带日期的本地
源码观察，不代表远端仓库的最新状态。

本 crate 面向 Rust 1.94 及以上版本，状态表示保持为 `u64`。相较于 `qubit-cas` 0.8
中的 `usize` 状态及 `qubit_atomic::Atomic<usize>` 类型别名，调用方可能需要转换状态值，
并将其他底层原子操作改用 `CasCell` 方法，或单独持有所需原子类型。`load`、`store`、
`swap` 和 `compare_set` 仍可使用。
