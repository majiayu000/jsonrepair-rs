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
| jsonrepair-rs | valid_small | 47 | 4.024 | 3.253 | 5.129 | 0.01 | ok |  |
| jsonrepair-rs | broken_small | 48 | 3.596 | 3.262 | 8.388 | 0.01 | ok |  |
| jsonrepair-rs | valid_large_1k | 46670 | 4.462 | 4.079 | 6.269 | 9.98 | ok |  |
| jsonrepair-rs | broken_large_1k | 47670 | 4.549 | 4.012 | 6.534 | 9.99 | ok |  |
| jsonrepair-rs | nested_100 | 200 | 3.813 | 3.375 | 6.228 | 0.05 | ok |  |
| jsonrepair-rs | comments_100 | 3188 | 3.720 | 3.329 | 10.425 | 0.82 | ok |  |
| jsonrepair-rs | string_escapes_200 | 5401 | 3.882 | 3.443 | 5.709 | 1.33 | ok |  |
| python-json-repair | valid_small | 47 | 54.629 | 51.719 | 80.348 | 0.00 | ok |  |
| python-json-repair | broken_small | 48 | 54.400 | 52.383 | 77.061 | 0.00 | ok |  |
| python-json-repair | valid_large_1k | 46670 | 56.819 | 55.352 | 68.502 | 0.78 | ok |  |
| python-json-repair | broken_large_1k | 47670 | 76.357 | 74.040 | 97.843 | 0.60 | ok |  |
| python-json-repair | nested_100 | 200 | 56.838 | 52.920 | 68.792 | 0.00 | ok |  |
| python-json-repair | comments_100 | 3188 | 55.749 | 52.972 | 124.548 | 0.05 | ok |  |
| python-json-repair | string_escapes_200 | 5401 | 54.871 | 52.594 | 90.096 | 0.09 | ok |  |

## Current Hotspots

Slowest median latency:
- `python-json-repair` / `broken_large_1k`: 76.357 ms
- `python-json-repair` / `nested_100`: 56.838 ms
- `python-json-repair` / `valid_large_1k`: 56.819 ms

Lowest throughput among inputs >= 1 KiB:
- `python-json-repair` / `comments_100`: 0.05 MiB/s
- `python-json-repair` / `string_escapes_200`: 0.09 MiB/s
- `python-json-repair` / `broken_large_1k`: 0.60 MiB/s
