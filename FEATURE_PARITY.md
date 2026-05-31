# Feature Parity

This page compares `jsonrepair-rs` with the established JavaScript
`jsonrepair` package and Python `json-repair` package.

The goal is practical adoption clarity, not a claim that every internal
heuristic is byte-for-byte identical. Repair status was checked on 2026-06-01
against the public upstream documentation plus the `jsonrepair-rs` README and
test suite. When a behavior is broad or implementation-dependent, the notes
call that out.

## Summary

| Area | JS `jsonrepair` | Python `json-repair` | `jsonrepair-rs` |
| --- | --- | --- | --- |
| Main language/runtime | JavaScript/TypeScript | Python | Rust |
| Primary return type | Repaired JSON string | Python object or repaired JSON string | Repaired JSON string |
| CLI | Yes | Yes | Yes |
| Typed/native parse helper | Use `JSON.parse` after repair | Yes, via `loads`, `load`, and `return_objects` | Yes, with the optional `serde` feature |
| Strict mode | Not documented | Yes | Yes |
| True low-memory streaming repair | Yes, Node transform API | Partial, stream-stable partial-output mode | No, current Rust API buffers internally |
| Schema-guided repair | Not documented | Yes, beta JSON Schema/Pydantic support | No |
| Browser/demo surface | Yes | Yes | No dedicated browser demo yet |

## Repair Behavior

| Behavior | JS `jsonrepair` | Python `json-repair` | `jsonrepair-rs` | Rust coverage |
| --- | --- | --- | --- | --- |
| Valid JSON pass-through | Yes | Yes | Yes | `tests/repair_tests.rs` valid JSON cases |
| Missing quotes around object keys | Yes | Yes | Yes | `unquoted_keys`, parity fixtures |
| Single quotes | Yes | Yes | Yes | `single_quotes_to_double` |
| Curly/special quotes | Yes | Not specifically documented | Yes | `curly_double_quotes`, `curly_single_quotes` |
| Backtick quoted strings | Not specifically documented | Not specifically documented | Yes | `backtick_quotes` |
| Missing escape characters and invalid escapes | Yes | Yes | Yes | string repair tests |
| Unescaped control characters in strings | Yes | Yes | Yes | string/control character tests |
| Missing commas | Yes | Yes | Yes | missing-comma tests and fixtures |
| Leading commas | Not specifically documented | Not specifically documented | Yes | leading-comma tests |
| Trailing commas | Yes | Yes | Yes | trailing-comma tests and fixtures |
| Missing closing brackets/braces | Yes | Yes | Yes | truncation tests and fixtures |
| Truncated JSON values and strings | Yes | Yes | Yes | truncation tests |
| Missing colons | Not specifically listed | Yes, broadly documented | Yes | missing-colon tests |
| Missing object values | Not specifically listed | Yes | Yes | missing-value tests |
| Comments (`//`, `/* */`) | Yes | Yes | Yes | comment tests and fixtures |
| Hash comments (`#`) | Not specifically documented | Comments broadly documented | Yes | `hash_comment` |
| Markdown fenced code blocks | Yes | LLM examples cover markdown/prose | Yes | markdown fence tests and fixtures |
| Python constants (`None`, `True`, `False`) | Yes | Yes | Yes | Python keyword tests |
| JavaScript non-standard values (`undefined`, `NaN`, `Infinity`) | Not specifically listed except broad JSON repair | Not specifically documented | Yes | JavaScript keyword tests |
| Signed non-finite values (`+NaN`, `-Infinity`) | Not specifically documented | Not specifically documented | Yes | signed non-finite tests |
| String concatenation (`"a" + "b"`) | Yes | Not specifically documented | Yes | string concatenation tests |
| JSONP wrappers | Yes | Not specifically documented | Yes | JSONP tests and fixtures |
| MongoDB wrappers | Yes | Not specifically documented | Yes | MongoDB tests and fixtures |
| NDJSON/root value lists | Yes | Not specifically documented | Yes | NDJSON/root list tests and fixtures |
| Ellipsis placeholders | Yes | Not specifically documented | Yes | ellipsis tests and fixtures |
| Escaped JSON strings | Yes | Escaping behavior documented | Yes | escaped string wrapper tests |
| URLs as unquoted strings | Not specifically documented | Stray prose/string cleanup documented | Yes | URL tests and fixtures |
| Regex-like tokens as strings | Not specifically documented | Not specifically documented | Yes | regex-like tests and fixtures |
| Redundant closing brackets | Not specifically documented | Not specifically documented | Yes | redundant closer tests |
| Special whitespace normalization | Yes | Not specifically documented | Yes | special whitespace tests |
| Safe failure on unrecoverable input | Throws `JSONRepairError` | Raises/returns failure depending on API | Returns `JsonRepairError` | non-repairable tests |

## API And Ecosystem Differences

| Capability | JS `jsonrepair` | Python `json-repair` | `jsonrepair-rs` |
| --- | --- | --- | --- |
| Package install | `npm install jsonrepair` | `pip install json-repair` | `cargo add jsonrepair-rs` |
| Default dependencies | npm package runtime | Python package runtime | No default Rust dependencies |
| Optional parse helpers | No, callers parse separately | Built in | `serde` feature |
| Writer API | Node stream writes | File/CLI helpers | `std::io::Write` helper |
| Reader-to-writer API | True streaming transform | File/CLI helpers and stream-stable option | IO convenience API, buffers internally |
| Error location metadata | Error object | Python exceptions/errors | Error kind, position, line, and column |
| Configurable policy | Stream buffer options | Strict/schema/formatting options | Strict mode only for now |
| Fuzzing harness | Not documented | Not documented | `fuzz/` plus regression tests |
| Compatibility corpus | Upstream tests | Python tests/examples | `tests/fixtures/parity_cases.json` |

## Known Gaps In `jsonrepair-rs`

- The reader-to-writer APIs are streaming-oriented at the IO boundary, but the
  current parser still reads the full input and repaired output into memory.
  See `docs/streaming-api.md`.
- There is no schema-guided repair mode like Python `json-repair`'s beta JSON
  Schema/Pydantic support.
- There is no browser/WASM playground or npm wrapper yet. See
  `docs/ecosystem-evaluations.md`.
- The parity fixture corpus is representative, not exhaustive. Add cases to
  `tests/fixtures/parity_cases.json` when a new upstream-style behavior matters.
- Strict mode currently rejects input when the repaired output differs from the
  original text. It is not a duplicate-key validator or schema validator.

## Sources

- JS `jsonrepair`: https://github.com/josdejong/jsonrepair
- Python `json-repair`: https://github.com/mangiucugna/json_repair
- Rust `jsonrepair-rs` repair tests: `tests/repair_tests.rs`
- Rust parity fixtures: `tests/fixtures/parity_cases.json`
- Rust streaming design: `docs/streaming-api.md`
