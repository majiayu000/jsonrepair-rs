# Streaming API Design

## Goal

Expose a stable Rust API for repairing JSON-like text from an input stream into
an output stream, so callers do not have to receive an owned repaired `String`
when they are already working with files, stdin/stdout, pipes, or other
`std::io` types.

## MVP API

```rust
pub fn jsonrepair_reader_to_writer<R, W>(
    reader: R,
    writer: &mut W,
) -> Result<(), JsonRepairStreamError>
where
    R: std::io::Read,
    W: std::io::Write + ?Sized;
```

The function accepts any sync `std::io::Read` and writes repaired JSON bytes to
any sync `std::io::Write`.

## Examples

File to file:

```rust,no_run
use std::fs::File;
use jsonrepair_rs::jsonrepair_reader_to_writer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = File::open("broken.json")?;
    let mut output = File::create("repaired.json")?;

    jsonrepair_reader_to_writer(&mut input, &mut output)?;
    Ok(())
}
```

stdin to stdout:

```rust,no_run
use std::io;
use jsonrepair_rs::jsonrepair_reader_to_writer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    jsonrepair_reader_to_writer(stdin.lock(), &mut stdout.lock())?;
    Ok(())
}
```

## Error Model

`JsonRepairStreamError` separates the three failure classes:

- `Read(std::io::Error)` when the source cannot be read as UTF-8 text.
- `Repair(JsonRepairError)` when the input cannot be repaired safely.
- `Write(std::io::Error)` when the destination cannot be written.

## Memory Behavior

The `0.2.0` MVP is streaming-oriented at the API boundary, but it is not yet a
constant-memory streaming parser.

Current behavior:

1. Read the complete input stream into an internal `String`.
2. Run the existing repair parser.
3. Buffer the repaired output internally.
4. Write the repaired JSON to the destination.

This keeps behavior identical to `jsonrepair(input)` and avoids exposing a
partially repaired output on repair failure. It also means peak memory is still
roughly proportional to input size plus repaired output size.

## Non-Goals For The MVP

- Async IO support.
- Partial JSON value emission before the full repair succeeds.
- Constant-memory repair for arbitrarily large inputs.
- Changing parser semantics or whitespace preservation.
- Supporting non-UTF-8 byte streams.

## Future Direction

A true streaming parser can preserve this high-level API while changing the
implementation underneath. The parser would need explicit rollback windows for
repairs such as trailing-comma removal and delayed delimiter insertion, plus a
clear policy for whether repair failures may have already written partial
output.

## True Streaming Parser Design

A lower-memory parser should move from "read all, repair all, write all" to an
incremental state machine with explicit commit points. The high-level
`jsonrepair_reader_to_writer` API can stay source-compatible, but the internal
parser needs to know when bytes are safe to emit.

Required parser states:

| State | Responsibility | Rollback window |
| --- | --- | --- |
| Whitespace/comment scanner | Copy whitespace, skip comments, preserve line info | Until a comment end or newline confirms what to drop. |
| String scanner | Normalize quotes, escapes, control characters, and missing end quotes | From opening quote through the next safe delimiter. |
| Number scanner | Repair `.5`, `2.`, `2e`, signed non-finite values | Until token boundary confirms the number shape. |
| Object scanner | Repair keys, colons, commas, values, and closing braces | At least one property plus trailing whitespace/comma. |
| Array scanner | Repair commas, values, ellipsis, and closing brackets | At least one item plus trailing whitespace/comma. |
| Root scanner | Detect NDJSON/root value lists, JSONP, markdown fences, redundant closers | Until the next root token proves list aggregation is needed. |

Commit policy:

1. Do not write partial output before a repair decision is irreversible.
2. Keep a small pending-output buffer per nesting frame.
3. Commit a frame only after delimiters, comments, and trailing commas are
   resolved.
4. On repair failure, return `JsonRepairStreamError::Repair` without writing
   partial JSON unless a future API explicitly opts into partial output.

Chunk-boundary tests should cover:

- strings and escapes split at every byte position
- comments split across `//`, `/*`, and `*/`
- numbers split across sign, decimal point, exponent, and non-finite keyword
- trailing commas split before and after whitespace
- missing delimiters at EOF
- NDJSON/root value aggregation across chunks

The current tests already assert that a chunked reader produces exactly the same
output as `jsonrepair(input)` for these categories. A parser rewrite should keep
those tests and add lower-level state-machine tests before changing memory
behavior.
