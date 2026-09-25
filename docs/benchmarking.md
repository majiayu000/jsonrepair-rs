# Benchmarking

`jsonrepair-rs` keeps two benchmark surfaces:

- Criterion benchmarks in `benches/benchmark.rs` for Rust-level parser timing.
- `scripts/benchmark_report.py` for CLI-level latency and throughput reports
  against Python `json-repair` and optional CLI competitors.

Normal CI should keep running tests and clippy only. Benchmark reports are for
manual release and optimization checks because local CPU load can easily change
results.

## Criterion Benchmarks

Run the built-in parser benchmarks:

```bash
cargo bench
```

The cases cover small valid and broken inputs, 1k-item valid and broken arrays,
deep nesting, comment-heavy objects, and string-escape-heavy arrays.

For optimization work, use the existing gate script:

```bash
scripts/opt_round.sh --baseline before-change
```

The gate script reruns noisy Criterion results and checks Criterion estimates
instead of trusting a single benchmark pass.

## CLI Throughput Report

Generate a Markdown report for local CLI timing:

```bash
python3 -m pip install json-repair
python3 scripts/benchmark_report.py \
  --adapters jsonrepair-rs,python-json-repair \
  --output docs/reports/benchmark-metrics.md
```

The `jsonrepair-rs` adapter is built in release mode from this checkout.
The Python adapter runs `python3 -m json_repair` with the same stdin input.
`llm-json` remains optional and is reported as `skipped` when absent.

The report includes:

- median, min, and max wall-clock latency per case
- input size
- approximate input throughput in MiB/s
- slowest median-latency cases
- lowest-throughput cases

These numbers include process startup, stdin/stdout handling, parsing, repair,
and output serialization. Python's CLI also pretty-prints by default. Outputs
are checked for valid JSON, but are not required to have the same repair
semantics. This is an end-to-end CLI comparison, not an in-process parser speed
claim. Use Criterion when measuring Rust parser-only changes. Record the OS,
CPU, Python package version, Rust version, iterations, and local load before
using results in a performance claim.

## Allocation Checks

Rust-level allocation counts are not stable enough to add to normal CI without a
pinned profiler environment. Use one of these tools during targeted performance
work:

| Environment | Command shape | Notes |
| --- | --- | --- |
| Linux heaptrack | `heaptrack target/release/jsonrepair < input.json` | Good first choice for allocation hot paths. |
| Linux Massif | `valgrind --tool=massif target/release/jsonrepair < input.json` | Slower, useful for peak heap profiles. |
| macOS Instruments | Profile `target/release/jsonrepair` with Allocations | Best local option on macOS. |

Use the same input cases from `scripts/benchmark_report.py` when profiling
allocations so latency and allocation reports describe the same workload.
