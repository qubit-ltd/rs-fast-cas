// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::convert::Infallible;
use std::hint::black_box;
use std::sync::Barrier;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_fast_cas::FastCas;
use qubit_fast_cas::FastCasError;
use qubit_fast_cas::FastCasState;

/// Number of successful updates completed by each worker in one measured
/// contention iteration.
const OPERATIONS_PER_THREAD: u64 = 4_096;

/// Benchmarks direct and policy-driven atomic updates without contention.
fn benchmark_uncontended(c: &mut Criterion) {
    let mut group = c.benchmark_group("uncontended");
    group.throughput(Throughput::Elements(1));

    let direct = AtomicU64::new(0);
    group.bench_function("atomic_u64", |b| {
        b.iter(|| black_box(direct.fetch_add(1, Ordering::AcqRel)));
    });

    let compare_state = FastCasState::new(0);
    let mut expected = 0u64;
    group.bench_function("compare_update", |b| {
        b.iter(|| {
            let next = expected.wrapping_add(1);
            let success = FastCas::once()
                .compare_update(&compare_state, expected, next)
                .expect("expected state should update");
            expected = next;
            black_box(success)
        });
    });

    for (name, cas) in fast_cas_policies() {
        let state = FastCasState::new(0);
        group.bench_function(name, |b| {
            b.iter(|| {
                let success = cas
                    .update_by(&state, |current| {
                        Ok::<(u64, u64), Infallible>((current.wrapping_add(1), current))
                    })
                    .expect("uncontended update should not conflict");
                black_box(success.into_output())
            });
        });
    }
    group.finish();
}

/// Benchmarks fixed successful-update workloads under shared-state contention.
fn benchmark_contended(c: &mut Criterion) {
    let mut group = c.benchmark_group("contended");
    for thread_count in [2usize, 4, 8] {
        let thread_count_u64 = u64::try_from(thread_count).expect("thread count should fit u64");
        group.throughput(Throughput::Elements(OPERATIONS_PER_THREAD * thread_count_u64));
        group.bench_with_input(
            BenchmarkId::new("atomic_u64", thread_count),
            &thread_count,
            |b, &threads| {
                b.iter_custom(|iterations| run_atomic_workload(threads, OPERATIONS_PER_THREAD, iterations));
            },
        );
        for (name, cas) in fast_cas_policies() {
            group.bench_with_input(BenchmarkId::new(name, thread_count), &thread_count, |b, &threads| {
                b.iter_custom(|iterations| run_fast_cas_workload(cas, threads, OPERATIONS_PER_THREAD, iterations));
            });
        }
    }
    group.finish();
}

/// Returns the retry policies measured by both benchmark groups.
fn fast_cas_policies() -> [(&'static str, FastCas); 3] {
    [
        ("once", FastCas::once()),
        ("spin_16", FastCas::spin(16)),
        ("spin_yield_8_64", FastCas::spin_yield(8, 64)),
    ]
}

/// Runs a direct `AtomicU64` workload and returns only its measured duration.
fn run_atomic_workload(thread_count: usize, operations_per_thread: u64, iterations: u64) -> Duration {
    let state = AtomicU64::new(0);
    run_workers(thread_count, || {
        let state = &state;
        let total_operations = operations_per_thread
            .checked_mul(iterations)
            .expect("benchmark operation count should fit u64");
        move || {
            for _ in 0..total_operations {
                black_box(state.fetch_add(1, Ordering::AcqRel));
            }
        }
    })
}

/// Runs a policy-driven workload and retries whole operations after conflicts.
fn run_fast_cas_workload(cas: FastCas, thread_count: usize, operations_per_thread: u64, iterations: u64) -> Duration {
    let state = FastCasState::new(0);
    run_workers(thread_count, || {
        let state = &state;
        let total_operations = operations_per_thread
            .checked_mul(iterations)
            .expect("benchmark operation count should fit u64");
        move || {
            for _ in 0..total_operations {
                loop {
                    match cas.update_by(state, |current| {
                        Ok::<(u64, ()), Infallible>((current.wrapping_add(1), ()))
                    }) {
                        Ok(success) => {
                            black_box(success);
                            break;
                        }
                        Err(FastCasError::Conflict { .. }) => {}
                        Err(FastCasError::Abort { error, .. }) => match error {},
                    }
                }
            }
        }
    })
}

/// Starts synchronized scoped workers and measures their complete execution.
fn run_workers<F, W>(thread_count: usize, worker_factory: F) -> Duration
where
    F: Fn() -> W,
    W: FnOnce() + Send,
{
    let start_barrier = Barrier::new(thread_count + 1);
    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            let worker = worker_factory();
            let start_barrier = &start_barrier;
            handles.push(scope.spawn(move || {
                start_barrier.wait();
                worker();
            }));
        }
        start_barrier.wait();
        let started = Instant::now();
        for handle in handles {
            handle.join().expect("benchmark worker should not panic");
        }
        started.elapsed()
    })
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_millis(800));
    targets = benchmark_uncontended, benchmark_contended
}
criterion_main!(benches);
