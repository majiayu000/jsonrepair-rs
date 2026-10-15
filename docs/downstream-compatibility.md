# Downstream compatibility boundaries

## Scope and provenance

OpenBitFun dependency evidence was refreshed from public source on 2026-10-15;
the wecom-cli and OpenFlow snapshots below were checked on 2026-10-05.
The regression cases in
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
- **OpenBitFun** now declares the crates.io `jsonrepair-rs` version requirement
  `0.2.6`, with default features disabled, in its
  [workspace manifest at `3145662`](https://github.com/GCWing/OpenBitFun/blob/3145662201e1ec64adcf01ac2f5badb2d8a4c6a3/Cargo.toml#L99).
  Its [agent-stream manifest](https://github.com/GCWing/OpenBitFun/blob/3145662201e1ec64adcf01ac2f5badb2d8a4c6a3/src/crates/execution/agent-stream/Cargo.toml#L18)
  uses the workspace dependency. These pinned public-main manifests were checked
  on 2026-10-15; the requirement is not an exact version pin or production
  deployment receipt. The earlier
  [fork at `18aa544`](https://github.com/GCWing/OpenBitFun/blob/18aa5441d89f5e24cfb4d8ff54d4b53d92c68896/src/crates/execution/tool-call-jsonrepair/README.md)
  is historical context for the comparison below, not the current dependency.
  The downstream maintainer [reported](https://github.com/GCWing/OpenBitFun/issues/3229#issuecomment-6061424116)
  that all 66 isolated agent-stream integration tests passed after substituting
  upstream `aab8317de132915d94da9ec980cec4b362758884`, including synthetic
  Markdown, escapes and Windows-path cases. We did not run that downstream
  suite; the report and manifest migration do not establish production acceptance.

## Comments: syntax repair can lose intended content

Outside quoted strings, the default API strips `#`, `//` and `/* ... */`
comments. Inside quoted strings it preserves those markers. Recognized URLs
are also covered by the regression corpus.

For example, the malformed input `{"content": # Release notes"}` repairs to
`{"content": null}` under the default policy. If the omitted opening quote was
an accident, the intended Markdown has been lost even though the output is
valid JSON. Strict mode rejects the input; it does not recover the text.

A historical independent core-library comparison used upstream `5786580`,
upstream tag `v0.2.1`, and the historical OpenBitFun fork at `18aa544`. All three were
compiled directly from their Rust library source without optional features;
this was not a downstream workspace build. The following synthetic examples
show the policy boundary:

| Input | Upstream 0.2.1 and `5786580` (default) | Historical OpenBitFun tool profile |
| --- | --- | --- |
| `{"content": # Release notes"}` | `{"content": null}` | Preserves `# Release notes` as a string |
| `{"content": // Release notes"}` | `{"content": null}` | Returns `ColonExpected` |
| `{"content": /* Release notes */}` | `{"content": null}` | Preserves `/* Release notes */` as a string |
| Properly quoted comment-like content | Preserved | Preserved |

The table above describes the earlier fork comparison, not the new opt-in
upstream policies. Following the
[OpenBitFun #3229 reply](https://github.com/GCWing/OpenBitFun/issues/3229#issuecomment-6030422809),
upstream 0.2.6 provides independent `with_preserve_comment_markers(true)` and
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

Additional crate regressions cover preserved comment markers alongside leading
regex values, URL `+` and `;`, unbalanced URL parentheses, and nested wrappers
containing object keys and ordinary colon-bearing values. Integration cases
exercise escape decoding within those values and strict-mode precedence over
both policies. These are synthetic crate tests, not a build or release
acceptance test of a downstream workspace. URL recognition uses raw prefixes;
escape decoding does not reinterpret an escaped prefix as URL syntax. See
[repair options](repair-options.md) for that boundary.

Before executing repaired tool arguments, check both the application schema and
important content invariants. Schema validation alone cannot establish that the
repaired text matches the sender's intent, especially if a field permits null.

The CI workflow is configured to run the existing options, option-boundary,
comment-policy and bounded-input suites on `windows-latest` with all features.
The new job awaits its first CI run; the historical native results above are
separate from that pending verification.

## Completion and resource boundaries

Repairing a transport prefix can produce plausible but premature JSON. Wait for
the application's completed-message signal before executing tool arguments.
The reader API waits for EOF and buffers the complete input; it is not a
progressive tool-call decoder. A socket that never reaches EOF does not finish
repairing, even if it has already delivered a complete JSON value.

Input bytes, the parser's `Vec<char>`, and repaired output can coexist. The
parser's character storage alone uses four bytes per Unicode scalar value,
excluding capacity overhead. With `serde` helpers, the parsed value adds further
allocations. See the [macOS memory measurement](reports/memory-20261015.md) for
measured scaling on public synthetic reproductions and amplified regression
shapes, not production payloads. A nesting limit is not a byte or memory limit.
Configure
`RepairOptions::with_max_input_bytes` for an input byte budget.
Readers consume at most `limit + 1` bytes and reject excess input rather than
repairing a silently truncated `take(limit)` prefix. Apply timeouts to untrusted
or potentially unending readers as well.

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

## Bounded completed-argument integration

The runnable `examples/tool_arguments.rs` uses the existing independent content
options plus `with_max_input_bytes`. It follows OpenBitFun's completed-string
repair call shape, then strictly parses the candidate. The tool owner must still
validate its schema and important content. For wecom-cli, a limit on the upstream
candidate cannot bound its earlier strict-parse or candidate-enumeration work;
its owner must apply a budget at that pipeline's input boundary too.

This is a synthetic integration example, not a downstream application migration.
The content/escape options were already delivered in PRs #84–#86; this change
adds the missing byte budget without replacing those parser changes. The old
standalone fork comparison does not prove every current fork difference is
removed. The public dependency migration and downstream-reported integration
result are recorded above; production acceptance remains unverified.
