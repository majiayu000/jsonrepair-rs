# Benchmark Metrics Report

Generated: 2026-04-30

This report times representative repair inputs through CLI adapters. It is intended
for local comparison and trend inspection, not as a stable CI gate.

Measured iterations per adapter/case: `10`

| Adapter | Case | Input bytes | Median ms | Min ms | Max ms | Throughput MiB/s | Status | Note |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| jsonrepair-rs | valid_small | 47 | 3.647 | 3.212 | 4.886 | 0.01 | ok |  |
| jsonrepair-rs | broken_small | 48 | 3.860 | 3.175 | 6.109 | 0.01 | ok |  |
| jsonrepair-rs | valid_large_1k | 46670 | 7.764 | 6.586 | 9.792 | 5.73 | ok |  |
| jsonrepair-rs | broken_large_1k | 47670 | 8.276 | 7.518 | 9.557 | 5.49 | ok |  |
| jsonrepair-rs | nested_100 | 200 | 4.181 | 3.196 | 7.918 | 0.05 | ok |  |
| jsonrepair-rs | comments_100 | 3188 | 3.388 | 3.062 | 4.732 | 0.90 | ok |  |
| jsonrepair-rs | string_escapes_200 | 5401 | 4.482 | 3.754 | 5.087 | 1.15 | ok |  |
| llm-json | valid_small | 47 | 0.000 | 0.000 | 0.000 | 0.00 | skipped | llm_json not found on PATH |
| llm-json | broken_small | 48 | 0.000 | 0.000 | 0.000 | 0.00 | skipped | llm_json not found on PATH |
| llm-json | valid_large_1k | 46670 | 0.000 | 0.000 | 0.000 | 0.00 | skipped | llm_json not found on PATH |
| llm-json | broken_large_1k | 47670 | 0.000 | 0.000 | 0.000 | 0.00 | skipped | llm_json not found on PATH |
| llm-json | nested_100 | 200 | 0.000 | 0.000 | 0.000 | 0.00 | skipped | llm_json not found on PATH |
| llm-json | comments_100 | 3188 | 0.000 | 0.000 | 0.000 | 0.00 | skipped | llm_json not found on PATH |
| llm-json | string_escapes_200 | 5401 | 0.000 | 0.000 | 0.000 | 0.00 | skipped | llm_json not found on PATH |

## Current Hotspots

Slowest median latency:
- `jsonrepair-rs` / `broken_large_1k`: 8.276 ms
- `jsonrepair-rs` / `valid_large_1k`: 7.764 ms
- `jsonrepair-rs` / `string_escapes_200`: 4.482 ms

Lowest throughput among inputs >= 1 KiB:
- `jsonrepair-rs` / `comments_100`: 0.90 MiB/s
- `jsonrepair-rs` / `string_escapes_200`: 1.15 MiB/s
- `jsonrepair-rs` / `broken_large_1k`: 5.49 MiB/s
