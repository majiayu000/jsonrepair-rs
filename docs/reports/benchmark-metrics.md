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
| jsonrepair-rs | valid_small | 47 | 3.167 | 2.766 | 4.723 | 0.01 | ok |  |
| jsonrepair-rs | broken_small | 48 | 2.856 | 2.762 | 3.213 | 0.02 | ok |  |
| jsonrepair-rs | valid_large_1k | 46670 | 3.834 | 3.313 | 4.252 | 11.61 | ok |  |
| jsonrepair-rs | broken_large_1k | 47670 | 3.874 | 3.577 | 4.123 | 11.73 | ok |  |
| jsonrepair-rs | nested_100 | 200 | 3.289 | 2.879 | 3.769 | 0.06 | ok |  |
| jsonrepair-rs | comments_100 | 3188 | 3.125 | 2.998 | 3.596 | 0.97 | ok |  |
| jsonrepair-rs | string_escapes_200 | 5401 | 3.297 | 2.988 | 5.130 | 1.56 | ok |  |
| python-json-repair | valid_small | 47 | 49.611 | 47.708 | 52.707 | 0.00 | ok |  |
| python-json-repair | broken_small | 48 | 49.745 | 48.043 | 52.603 | 0.00 | ok |  |
| python-json-repair | valid_large_1k | 46670 | 52.229 | 50.495 | 55.476 | 0.85 | ok |  |
| python-json-repair | broken_large_1k | 47670 | 70.675 | 69.538 | 72.851 | 0.64 | ok |  |
| python-json-repair | nested_100 | 200 | 52.263 | 48.345 | 64.724 | 0.00 | ok |  |
| python-json-repair | comments_100 | 3188 | 50.118 | 49.269 | 54.241 | 0.06 | ok |  |
| python-json-repair | string_escapes_200 | 5401 | 50.455 | 47.510 | 53.655 | 0.10 | ok |  |

## Current Hotspots

Slowest median latency:
- `python-json-repair` / `broken_large_1k`: 70.675 ms
- `python-json-repair` / `nested_100`: 52.263 ms
- `python-json-repair` / `valid_large_1k`: 52.229 ms

Lowest throughput among inputs >= 1 KiB:
- `python-json-repair` / `comments_100`: 0.06 MiB/s
- `python-json-repair` / `string_escapes_200`: 0.10 MiB/s
- `python-json-repair` / `broken_large_1k`: 0.64 MiB/s
