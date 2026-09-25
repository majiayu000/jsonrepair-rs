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
| jsonrepair-rs | valid_small | 47 | 4.057 | 3.570 | 4.696 | 0.01 | ok |  |
| jsonrepair-rs | broken_small | 48 | 4.330 | 3.569 | 7.132 | 0.01 | ok |  |
| jsonrepair-rs | valid_large_1k | 46670 | 5.862 | 4.815 | 7.080 | 7.59 | ok |  |
| jsonrepair-rs | broken_large_1k | 47670 | 5.418 | 4.859 | 6.934 | 8.39 | ok |  |
| jsonrepair-rs | nested_100 | 200 | 5.086 | 3.835 | 7.165 | 0.04 | ok |  |
| jsonrepair-rs | comments_100 | 3188 | 5.069 | 4.512 | 7.752 | 0.60 | ok |  |
| jsonrepair-rs | string_escapes_200 | 5401 | 5.172 | 4.772 | 8.525 | 1.00 | ok |  |
| python-json-repair | valid_small | 47 | 78.579 | 63.960 | 120.434 | 0.00 | ok |  |
| python-json-repair | broken_small | 48 | 72.873 | 61.816 | 200.610 | 0.00 | ok |  |
| python-json-repair | valid_large_1k | 46670 | 74.998 | 63.668 | 123.220 | 0.59 | ok |  |
| python-json-repair | broken_large_1k | 47670 | 98.135 | 85.935 | 179.924 | 0.46 | ok |  |
| python-json-repair | nested_100 | 200 | 68.454 | 62.288 | 182.102 | 0.00 | ok |  |
| python-json-repair | comments_100 | 3188 | 70.190 | 59.452 | 126.678 | 0.04 | ok |  |
| python-json-repair | string_escapes_200 | 5401 | 65.340 | 59.311 | 116.343 | 0.08 | ok |  |

## Current Hotspots

Slowest median latency:
- `python-json-repair` / `broken_large_1k`: 98.135 ms
- `python-json-repair` / `valid_small`: 78.579 ms
- `python-json-repair` / `valid_large_1k`: 74.998 ms

Lowest throughput among inputs >= 1 KiB:
- `python-json-repair` / `comments_100`: 0.04 MiB/s
- `python-json-repair` / `string_escapes_200`: 0.08 MiB/s
- `python-json-repair` / `broken_large_1k`: 0.46 MiB/s
