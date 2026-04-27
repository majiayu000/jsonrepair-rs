# Competitor Comparison

`jsonrepair-rs` keeps a representative parity corpus in
`tests/fixtures/parity_cases.json`. The corpus is used by Rust tests and can
also be replayed against nearby JSON repair libraries.

## Generate A Report

Run the local crate plus any available optional competitors:

```bash
python3 scripts/compare_competitors.py \
  --adapters jsonrepair-rs,js-jsonrepair,python-json-repair,llm-json \
  --output docs/reports/competitor-comparison.md
```

The script always supports the `jsonrepair-rs` adapter. Other adapters are
optional:

| Adapter | Source | Requirement |
| --- | --- | --- |
| `jsonrepair-rs` | This repository's CLI | `cargo build --bin jsonrepair` |
| `js-jsonrepair` | `josdejong/jsonrepair` | `npx` |
| `python-json-repair` | Python `json-repair` | importable `json_repair` package |
| `llm-json` | Rust `llm_json` | `llm_json` binary on `PATH` |

Missing optional adapters are reported as `skipped` instead of failing the
report.

## Report Statuses

| Status | Meaning |
| --- | --- |
| `exact` | Output matches the fixture target string exactly. |
| `semantic` | Output parses to the same JSON value but formatting differs. |
| `different` | Output differs semantically or cannot be parsed as JSON. |
| `error` | The adapter returned a non-zero status or timed out. |
| `skipped` | The requested adapter was not available locally. |

Use `--fail-on-difference` when the report is part of a manual compatibility
gate. Do not add that mode to normal CI unless the competitor toolchain is
pinned and installed by the workflow.

## Adding Cases

Add new cases to `tests/fixtures/parity_cases.json` first. Keep the target
fixture value as the `jsonrepair-rs` output. If a divergence is intentional, set the
case's `divergence` object with a short `name` and `reason`; the Rust fixture
test validates that metadata.
