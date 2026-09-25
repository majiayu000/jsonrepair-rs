# Fuzzing

This repository includes two cargo-fuzz targets. `repair_parser` exercises the
public string repair entry point. `repair_stream` compares chunked reader-to-writer
repair with the string API and checks that repair failures write no partial output.
The targets run for 60 seconds each in the scheduled and manually triggered
`.github/workflows/fuzz.yml` workflow.

## Prerequisites

Install cargo-fuzz if it is not already available:

```sh
cargo install cargo-fuzz
```

## Build the fuzz target

```sh
cargo +nightly fuzz build repair_parser
cargo +nightly fuzz build repair_stream
```

## Run the fuzz target

```sh
cargo +nightly fuzz run repair_parser
cargo +nightly fuzz run repair_stream
```

For a short local smoke run, cap the number of generated inputs:

```sh
cargo +nightly fuzz run repair_parser -- -runs=1000
cargo +nightly fuzz run repair_stream -- -runs=1000
```

The target feeds arbitrary bytes through the `jsonrepair` entry point as lossy
UTF-8 text and treats any panic as a bug. When repair succeeds, the repaired
output must parse as `serde_json::Value`. The fuzz harness enables
`serde_json`'s `arbitrary_precision` and `unbounded_depth` features so valid
large exponents and container nesting within the repairer's depth limit are
not rejected by the validator's numeric or default depth limits. The stream
target uses arbitrary text
and chunk sizes from 1 to 32 bytes. It checks exact output equivalence on
success and the `Repair` error with empty output on repair failure.

If a crash is discovered, reduce the failing input with cargo-fuzz and add a
regression test under `tests/repair_tests.rs` before fixing the parser.
