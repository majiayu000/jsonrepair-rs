# Downstream compatibility boundaries

## Scope and provenance

Checked against public source on 2026-10-05. The regression cases in
`tests/downstream_contract_tests.rs` are newly written synthetic examples based
on the shapes handled at these call sites, not captured user payloads or copied
downstream test suites. They exercise this crate's public APIs, not the full
applications. No production payload corpus or end-to-end compatibility claim is
implied.

- **wecom-cli** at
  [`c4b9b66`](https://github.com/WecomTeam/wecom-cli/blob/c4b9b6610c7ca2854441bfa336a5b458daeeb707/crates/wecom/src/service/command/json_repair/mod.rs):
  strict `serde_json` first, then quote-repair candidates and
  `jsonrepair_value`. Schema-declared keys rank candidates; schema validation is
  left to the backend. Its manifest allows `0.2` with `serde`, while its checked
  lockfile resolves `0.2.1`. Tests here cover unquoted keys, embedded quotes,
  trailing commas and Python-like values. They do not reproduce candidate
  scoring or prove that this crate's candidate wins.
- **OpenFlow** at
  [`da6f24d`](https://github.com/philbotar/OpenFlow/blob/da6f24da3a1d2a47f66ce8c9fe6220e0a209c3eb/crates/providers/src/mapping/mod.rs):
  strict parse first, then `jsonrepair` and another strict parse, only if the
  trimmed input starts with `{` or `[`. Both manifest and checked lockfile refer
  to `0.2.1` (the manifest's Cargo version requirement is not an exact pin).
  Tests here cover incomplete paths, arrays, Unicode text and a trailing comma
  in a tool-result envelope. Application-level shape checks and repair admission
  remain OpenFlow's responsibility; this library also accepts scalar strings.
- **OpenBitFun** at
  [`18aa544`](https://github.com/GCWing/OpenBitFun/blob/18aa5441d89f5e24cfb4d8ff54d4b53d92c68896/src/crates/execution/tool-call-jsonrepair/README.md):
  a local MIT-licensed fork of `0.2.1`, called through `repair_tool_call_json` by
  its streamed tool-call accumulator. Its profile disables comment parsing.
  This is a different policy from upstream's generic repair API, not evidence
  that replacing the fork with upstream is drop-in compatible.

## Comments: syntax repair can lose intended content

Outside quoted strings, the default API strips `#`, `//` and `/* ... */`
comments. Inside quoted strings it preserves those markers. Recognized URLs
are also covered by the regression corpus.

For example, the malformed input `{"content": # Release notes"}` repairs to
`{"content": null}` under the default policy. If the omitted opening quote was
an accident, the intended Markdown has been lost even though the output is
valid JSON. Strict mode rejects the input; it does not recover the text.

A small independent core-library comparison used current upstream `5786580`,
upstream tag `v0.2.1`, and the OpenBitFun fork at the commit above. All three were
compiled directly from their Rust library source without optional features;
this was not a downstream workspace build. The following synthetic examples
show the policy boundary:

| Input | Upstream 0.2.1 and current main | OpenBitFun tool profile |
| --- | --- | --- |
| `{"content": # Release notes"}` | `{"content": null}` | Preserves `# Release notes` as a string |
| `{"content": // Release notes"}` | `{"content": null}` | Returns `ColonExpected` |
| `{"content": /* Release notes */}` | `{"content": null}` | Preserves `/* Release notes */` as a string |
| Properly quoted comment-like content | Preserved | Preserved |

The table above describes the earlier fork comparison, not the new opt-in
upstream policies. Following the
[OpenBitFun #3229 reply](https://github.com/GCWing/OpenBitFun/issues/3229#issuecomment-6030422809),
upstream now provides independent `with_preserve_comment_markers(true)` and
`with_decode_unquoted_escapes(true)` options. Both default to false. The option
tests cover all four combinations, the reply's escaped Markdown example, and
literal Windows paths across the string, writer, reader and serde helpers. The
fork's reported path corruption was not re-run as part of this implementation.
See [repair options](repair-options.md) for examples and boundaries.

The 11 option regression tests also passed natively on Windows on 2026-10-08
with Rust 1.99.0 (`x86_64-pc-windows-msvc`), using
`cargo test --locked --test options_tests --all-features`. This checks the new
upstream options on Windows, not the downstream fork or the full application.

Expanded native Windows testing on the same date adds nine tests in
`tests/options_boundary_tests.rs`: literal drive and UNC paths, CRLF, escaped
token delimiters, quoted control characters, transport prefixes and one-byte
reader chunks, surrogate-pair boundaries, and long tool content. Generated
cases cover 25,088 malformed-input/policy combinations and every BMP Unicode
code unit with escape decoding both enabled and disabled. Successful malformed
token repairs must parse as JSON and be idempotent; string and writer results
must agree without writing output on repair errors. Transport-prefix cases also
check the reader API and chunk boundaries. These finite synthetic cases do not
establish intent preservation for arbitrary malformed input.

The expanded tests found a pre-existing loss of a leading backslash in unquoted
object keys, including UNC paths. Quoted-key probing now consumes a backslash
only when it precedes a quote; the fix is covered across all policy combinations.
Run the expanded corpus with:

```sh
cargo test --locked --test options_boundary_tests --all-features -- --nocapture
```

After the key fix, the complete native Windows suite passed with 278 tests under
default features and 294 under all features, plus one all-feature doctest and
`cargo check --locked --all-targets --all-features`. The same source passed the
macOS suite, formatting, warning-denying compilation, Clippy, documentation,
package verification, and the Rust 1.70 core library/CLI check.

Disabling comment parsing does not guarantee recovery of every comment-like
unquoted value. Do not advertise universal content preservation for either
profile. Default comment behavior remains unchanged. Actual multiline unquoted
strings remain a separate issue; neither option enables them.

Before executing repaired tool arguments, check both the application schema and
important content invariants. Schema validation alone cannot establish that the
repaired text matches the sender's intent, especially if a field permits null.

## Completion and resource boundaries

Repairing a transport prefix can produce plausible but premature JSON. Wait for
the application's completed-message signal before executing tool arguments.
The reader API waits for EOF and buffers the complete input; it is not a
progressive tool-call decoder. A socket that never reaches EOF does not finish
repairing, even if it has already delivered a complete JSON value.

Input bytes, the parser's `Vec<char>`, and repaired output can coexist. The
parser's character storage alone uses four bytes per Unicode scalar value,
excluding capacity overhead. With `serde` helpers, the parsed value adds further
allocations. A nesting limit is not a byte or memory limit. Enforce an input-byte
budget before calling repair; for a stream, read at most `limit + 1` bytes and
reject over-limit input rather than repairing a silently truncated `take(limit)`
prefix. Apply timeouts to untrusted or potentially unending readers as well.

Read and repair failures happen before any writer calls. A write failure can
leave a prefix in the destination; use a temporary file and replace the target
only after success when atomic file replacement is required. See
[the IO design](streaming-api.md) for details.

## Schema correction is not validation

`jsonrepair_value_with_schema` performs selected value corrections through
`properties` and `items`. In particular, `Ok` does not establish compliance with
`required`, `additionalProperties`, numeric bounds, string constraints, `$ref`,
or union/combinator schemas. Ambiguous case-insensitive enum matches stay
unchanged, null is not wrapped into an array, and unsupported corrections can
leave a value with the wrong type. The boundary regression deliberately returns
an `Ok` value that fails several schema constraints. Validate separately before
using it.

## Running the corpus

```sh
cargo test --test downstream_contract_tests
cargo test --test downstream_contract_tests --features serde
```

The shared cases check parsed values, repair idempotence, strict acceptance of
repaired output, writer parity, and reader chunks of one through five bytes
(including splits inside UTF-8). With `serde`, they also check `jsonrepair_value`.
Separate cases assert read-failure output isolation, possible partial writes,
and schema-helper limits. Chunk equivalence is a behavior test, not a
constant-memory measurement.
