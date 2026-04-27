# Fuzzing

This repository includes a cargo-fuzz target for the public repair parser entry
point.

## Prerequisites

Install cargo-fuzz if it is not already available:

```sh
cargo install cargo-fuzz
```

## Build the fuzz target

```sh
cargo +nightly fuzz build repair_parser
```

## Run the fuzz target

```sh
cargo +nightly fuzz run repair_parser
```

For a short local smoke run, cap the number of generated inputs:

```sh
cargo +nightly fuzz run repair_parser -- -runs=1000
```

The target feeds arbitrary bytes through the `jsonrepair` entry point as lossy
UTF-8 text and treats any panic as a bug. When repair succeeds, the repaired
output must parse as `serde_json::Value`.

If a crash is discovered, reduce the failing input with cargo-fuzz and add a
regression test under `tests/repair_tests.rs` before fixing the parser.
