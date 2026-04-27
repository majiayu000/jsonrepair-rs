# Parity Fixtures

`parity_cases.json` stores representative repair cases for behavior parity
tracking against upstream-style `jsonrepair` inputs.

Each case has:

- `name`: stable case identifier.
- `category`: broad repair category.
- `input`: malformed or valid JSON-like input.
- `expected`: expected `jsonrepair-rs` output.
- `source`: where the case came from, such as `upstream-representative` or
  `project-regression`.
- `divergence`: `null` when behavior is expected to match upstream-style
  behavior. When behavior intentionally differs, set this to an object with a
  short `name` and `reason`.

The Rust fixture test validates every case and requires intentional divergences
to be named and justified.
