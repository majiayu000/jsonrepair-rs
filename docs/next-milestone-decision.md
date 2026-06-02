# Next Milestone Decision

Date: 2026-06-03

## Decision

Defer both true low-memory streaming repair and schema-guided repair for the
next implementation milestone. The next milestone should stay
compatibility-first: use the expanded parity corpus and checked-in competitor
report to choose small, testable behavior and policy slices before starting a
parser rewrite or schema semantics.

## Drivers

- Preserve the current reliability baseline. The crate now has unit tests,
  CLI tests, writer/reader tests, serde tests, fuzz coverage, and a 36-case
  upstream-style parity corpus.
- Avoid silent data changes. Repair must keep returning explicit errors for
  unrecoverable input instead of inventing schema-shaped values.
- Keep public API changes source-compatible and easy to explain.
- Prefer evidence from real corpus/report gaps over broad feature parity labels.

## Evidence

`FEATURE_PARITY.md` identifies two visible gaps:

- true low-memory streaming repair is available in the JavaScript package, while
  `jsonrepair-rs` still buffers internally;
- schema-guided repair exists in Python `json-repair` as beta functionality,
  while `jsonrepair-rs` has no schema mode.

`docs/streaming-api.md` and `src/lib.rs` show that the current reader-to-writer
API reads the full input, repairs it through the existing parser, then writes
the repaired output. That API shape is stable, but changing memory behavior
requires parser architecture work and clear partial-output policy.

`docs/llm-fallback-parsing.md` recommends typed `serde` parsing and
application-layer validation after repair. That keeps schema responsibility with
the caller today and avoids pretending that repair is validation.

The refreshed competitor report in
`docs/reports/competitor-comparison.md` compares the expanded 36-case corpus:

| Adapter | exact | semantic | different | error | skipped |
| --- | ---: | ---: | ---: | ---: | ---: |
| `jsonrepair-rs` | 36 | 0 | 0 | 0 | 0 |
| `js-jsonrepair` | 34 | 0 | 1 | 1 | 0 |
| `python-json-repair` | 10 | 9 | 17 | 0 | 0 |
| `llm-json` | 0 | 0 | 0 | 0 | 36 |

The report shows that compatibility evidence is now concrete enough to drive
smaller follow-up work, but it does not yet prove that true streaming or
schema-guided repair is the highest-impact next investment.

## Alternatives Considered

### True low-memory streaming parser

This would improve CLI and large-file workflows and close a visible JavaScript
parity gap. It is also the highest-risk option because the existing parser owns
the full input and repaired output. Current repair behavior sometimes needs
rollback or delayed decisions for strings, comments, trailing commas, NDJSON
aggregation, markdown fences, and redundant closers.

If this becomes the priority later, the smallest testable slice should be an
internal experimental scanner that:

- handles flat objects and arrays with strings, comments, numbers, and trailing
  commas;
- preserves the existing public reader-to-writer API;
- does not emit partial output on repair failure;
- proves equivalence against `jsonrepair(input)` for chunk boundaries.

Verification would start with `cargo test --test streaming_tests`, lower-level
state-machine tests, allocation or peak-memory measurement, and
`cargo test --all-targets --all-features`.

### Schema-guided repair

This would help LLM structured-output workflows and move closer to Python
`json-repair`'s beta schema surface. It is also a new product semantics layer,
not just parser repair. The main risk is silently filling, coercing, or
reshaping data in a way callers mistake for validation.

If this becomes the priority later, the supported subset should be explicit:

- JSON Schema `type`;
- object `properties` and `required`;
- array `items`;
- primitive scalar validation;
- `enum`.

Non-goals for the first slice should include `$ref`, `oneOf`/`anyOf`/`allOf`,
format validation, default-value insertion, Pydantic integration, and any mode
that silently fabricates missing required data. The feature should be behind an
optional dependency or explicit feature flag if it needs schema parsing.

Verification would require schema-subset tests, error-path tests proving no
silent fallback, serde helper regressions, and
`cargo test --all-targets --all-features`.

### Compatibility-first milestone

This is the chosen path. It keeps the next work narrow and evidence-driven.
The expanded corpus and report should be used to select small behavior or
policy improvements that can be reviewed independently.

The first implementation slice should be one of:

- classify the report's `different` and `error` rows into accepted divergence,
  candidate parity fix, or external-adapter limitation;
- add narrowly scoped fixture metadata for accepted divergences that are not yet
  documented;
- add one `RepairOptions` policy toggle only after its default compatibility
  behavior and representative tests are clear.

Each slice should be a small PR with targeted tests and an updated report when
the corpus changes.

## Revisit Criteria

Revisit true streaming when at least one of these is true:

- users report memory pressure or failure on large files;
- benchmark or allocation reports show memory as the adoption bottleneck;
- CLI/file workflows become a bigger priority than repair-surface compatibility;
- a prototype proves chunk-boundary equivalence without partial-output leaks.

Revisit schema-guided repair when at least one of these is true:

- users request schema-aware LLM output repair and can describe the schema
  subset they need;
- the crate has a clear policy for coercion, missing required fields, and
  validation errors;
- the implementation can avoid silent defaults and preserve explicit failure;
- optional dependency and feature-flag boundaries are designed.

## Consequences

- The public API remains stable for now.
- The project avoids a large parser rewrite before there is usage evidence.
- The project avoids schema semantics that could blur repair and validation.
- The competitor report becomes an input to roadmap decisions rather than only a
  documentation artifact.

## Follow-ups

- Track report-row classification as a small compatibility issue.
- Keep the competitor report refreshed when parity fixtures change.
- Keep true streaming and schema-guided repair documented as known gaps until
  one of the revisit criteria is met.
