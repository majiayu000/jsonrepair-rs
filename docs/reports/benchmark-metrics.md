# Benchmark Metrics Report

Generated: 2026-09-25

This report times representative repair inputs through CLI adapters. It is intended
for local comparison and trend inspection, not as a stable CI gate.
Each invocation starts a new process; results include Python interpreter and Rust CLI startup,
stdin/stdout, repair, and JSON serialization. They are not library-call timings.
Adapters can choose different valid repairs; this report checks JSON validity, not semantic parity.
System load, CPU time, and peak memory were not measured.

Platform: `macOS-26.6.2-arm64-arm-64bit`; CPU: `Apple M1 Pro (8 cores)`; Python: `3.11.2`; json-repair: `0.55.1`
Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
Rust adapter: optimized `cargo build --release` binary from this checkout.

Measured iterations per adapter/case: `20`

| Adapter | Case | Input bytes | Median ms | Min ms | Max ms | Throughput MiB/s | Status | Note |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| jsonrepair-rs | valid_small | 47 | 3.683 | 3.282 | 4.126 | 0.01 | ok |  |
| jsonrepair-rs | broken_small | 48 | 4.149 | 3.463 | 5.767 | 0.01 | ok |  |
| jsonrepair-rs | valid_large_1k | 46670 | 4.533 | 4.115 | 7.613 | 9.82 | ok |  |
| jsonrepair-rs | broken_large_1k | 47670 | 4.696 | 4.033 | 5.426 | 9.68 | ok |  |
| jsonrepair-rs | nested_100 | 200 | 4.037 | 3.661 | 6.912 | 0.05 | ok |  |
| jsonrepair-rs | comments_100 | 3188 | 4.281 | 3.700 | 28.848 | 0.71 | ok |  |
| jsonrepair-rs | string_escapes_200 | 5401 | 5.008 | 4.049 | 23.900 | 1.03 | ok |  |
| python-json-repair | valid_small | 47 | 60.453 | 57.596 | 76.492 | 0.00 | ok |  |
| python-json-repair | broken_small | 48 | 61.253 | 57.807 | 211.183 | 0.00 | ok |  |
| python-json-repair | valid_large_1k | 46670 | 67.295 | 63.998 | 82.545 | 0.66 | ok |  |
| python-json-repair | broken_large_1k | 47670 | 89.000 | 82.153 | 300.765 | 0.51 | ok |  |
| python-json-repair | nested_100 | 200 | 64.433 | 58.452 | 72.661 | 0.00 | ok |  |
| python-json-repair | comments_100 | 3188 | 67.027 | 57.825 | 155.210 | 0.05 | ok |  |
| python-json-repair | string_escapes_200 | 5401 | 69.739 | 58.948 | 128.530 | 0.07 | ok |  |

## Current Hotspots

Slowest median latency:
- `python-json-repair` / `broken_large_1k`: 89.000 ms
- `python-json-repair` / `string_escapes_200`: 69.739 ms
- `python-json-repair` / `valid_large_1k`: 67.295 ms

Lowest throughput among inputs >= 1 KiB:
- `python-json-repair` / `comments_100`: 0.05 MiB/s
- `python-json-repair` / `string_escapes_200`: 0.07 MiB/s
- `python-json-repair` / `broken_large_1k`: 0.51 MiB/s
