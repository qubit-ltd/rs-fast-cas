# Fast CAS benchmark measurement (2026-11-12)

## Setup

- Revision: `2c258a77aa744b2e2d5049c1db2590e12e79755f`, with the task changes
  present in the working tree at measurement time.
- Toolchain: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, `cargo 1.94.0`.
- Host: x86_64 Linux, Intel Xeon E5-2680 v4 @ 2.40 GHz; 56 logical CPUs
  reported by the host.
- Workload: 4,096 successful operations per worker per measured iteration;
  uncontended group and 2, 4, and 8 worker contention groups.
- Criterion configuration: 30 samples, 1 second warm-up, 2 second measurement
  window, as overridden on the command line.
- Command: `cargo bench --locked --bench fast_cas_bench -- --sample-size 30
  --warm-up-time 1 --measurement-time 2`.
- Criterion used its plotters backend because `gnuplot` was not installed.
- Contended samples include releasing the start barrier, all worker work, and
  joining workers. Thread creation and waiting at the ready barrier are outside
  the measured interval.

The intervals below are Criterion's reported time ranges, with its point
estimate in parentheses. Units are nanoseconds for the uncontended group and
microseconds for the contended groups.

## Results

| Group | AtomicU64 | Once | Spin(16) | SpinYield(8, 64) | Compare update |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uncontended | 9.7815–9.8048 (9.7934) ns | 11.470–11.518 (11.493) ns | 12.394–14.177 (13.178) ns | 11.483–11.561 (11.518) ns | 16.002–16.094 (16.038) ns |
| 2 workers | 133.90–137.26 (135.55) µs | 324.29–340.87 (332.72) µs | 313.09–390.28 (360.03) µs | 275.79–365.66 (324.71) µs | — |
| 4 workers | 287.85–299.71 (294.33) µs | 834.74–867.75 (851.32) µs | 661.92–976.21 (813.65) µs | 965.27–1,042.0 (1,005.2) µs | — |
| 8 workers | 601.38–622.44 (612.15) µs | 1,797.9–1,981.1 (1,876.7) µs | 1,835.1–2,087.1 (1,967.0) µs | 2,013.9–2,088.6 (2,050.1) µs | — |

The output reported outliers in each uncontended group (6.67–13.33%) and most
contended groups; the largest reported fraction was 23.33% for `Spin(16)` at
four workers. Some contended strategy intervals are wide and overlap. These
measurements compare the current implementations under this benchmark protocol;
they do not isolate retry or inlining as a single causal variable.

The corrected start protocol addresses workers running before the timer starts.
The measurement still includes barrier-release and join overhead, so these are
workload measurements rather than pure atomic-operation latency. This run does
not compare the corrected protocol with a historical baseline. It therefore
does not overturn the historical spin or inline decisions recorded in the
[release-hardening plan](../docs/superpowers/plans/2026-07-14-fast-cas-release-hardening.md).
No production strategy was changed based on this run.
