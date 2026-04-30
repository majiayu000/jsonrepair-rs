# Ecosystem Evaluations

This note covers the post-`0.2.0` expansion surfaces that should stay separate
from the core Rust crate until there is clear user demand.

## Summary

| Surface | Recommendation | First step |
| --- | --- | --- |
| WASM/npm binding | Defer implementation, keep design ready | Add a thin `wasm-bindgen` crate only after JS users ask for browser/npm usage. |
| Python binding | Defer implementation, do not compete with Python `json-repair` yet | Revisit after CLI/library adoption shows Python demand. |
| Binary distribution | Proceed incrementally | Add GitHub Release archives for the existing `jsonrepair` binary before Homebrew or npm-style installers. |

The core parser should remain in the current Rust crate. Any language binding
should be a thin wrapper around the stable `jsonrepair(input)` API first.

## WASM/npm Binding

Recommendation: defer for now.

The likely package shape is a separate wrapper crate, for example
`crates/jsonrepair-wasm`, that depends on the core crate and exposes a small
`wasm-bindgen` API:

```rust
#[wasm_bindgen(js_name = repair)]
pub fn repair(input: &str) -> Result<String, JsValue>
```

The npm package name should avoid pretending to own the generic `jsonrepair`
namespace. Prefer one of:

- `jsonrepair-rs`
- `@jsonrepair-rs/wasm`
- `@majiayu000/jsonrepair-rs`

Minimum build/test work before publishing:

- build with `wasm-pack` or direct `wasm-bindgen` tooling
- test Node usage
- test browser bundler usage
- verify TypeScript definitions
- document that the API returns repaired JSON text, not parsed JS objects

Maintenance cost:

- npm publishing credentials and package ownership
- generated JS/wasm artifact review
- browser and Node compatibility matrix
- separate semver expectations from the Rust crate

Rationale: the official `wasm-bindgen` guide supports exporting Rust functions
to JavaScript, and `wasm-pack` can package Rust-generated WebAssembly for npm,
but this repo has not yet seen JS consumer demand. The current JS ecosystem also
already has the original `jsonrepair` package, so a wrapper needs a clear
reason to exist beyond "Rust can compile to WASM."

## Python Binding

Recommendation: defer for now.

The likely package shape is a separate wrapper crate or workspace member, for
example `crates/jsonrepair-python`, built with PyO3 and maturin:

```python
import jsonrepair_rs

fixed = jsonrepair_rs.repair("{name: 'Ada', active: True}")
```

The Python package name should avoid colliding with the existing `json-repair`
package. Prefer one of:

- `jsonrepair-rs`
- `jsonrepair_rs`
- `rust-jsonrepair`

Minimum build/test work before publishing:

- PyO3 wrapper around `jsonrepair(input)`
- maturin build and publish workflow
- wheels for Linux, macOS, and Windows
- tests for `str -> str` repair behavior
- tests for Unicode, large inputs, and repair errors crossing the FFI boundary
- README that compares the Rust-backed wrapper with existing Python options

Maintenance cost:

- Python package naming and PyPI ownership
- wheel matrix across CPython versions and platforms
- native-extension support burden
- potential confusion with the mature Python `json-repair` package

Rationale: PyO3 is the standard path for native Python modules written in Rust,
and maturin is the common build/publish tool for PyO3 packages. That path is
feasible, but Python users already have a popular native Python package. This
binding should wait until there is a specific need for Rust behavior, speed, or
cross-language parity.

## Binary Distribution

Recommendation: proceed incrementally.

The next release should add GitHub Release archives for the existing
`jsonrepair` CLI binary. Do not start with a Homebrew tap or custom installer
before there is a stable binary artifact layout.

Initial platforms:

- `x86_64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Recommended artifact shape:

```text
jsonrepair-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
jsonrepair-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz.sha256
jsonrepair-vX.Y.Z-x86_64-pc-windows-msvc.zip
jsonrepair-vX.Y.Z-x86_64-pc-windows-msvc.zip.sha256
```

Minimum CI/release automation:

- build release binaries on tag push
- upload archives and SHA-256 files to the GitHub Release
- run `jsonrepair --version` and one stdin repair smoke test per target
- document `cargo install jsonrepair-rs` as the fallback path
- optionally add `package.metadata.binstall` after the artifact names are stable

Recommended first implementation:

1. Add a release workflow that builds and uploads archives/checksums for the
   five initial targets.
2. Keep `cargo install jsonrepair-rs` as the primary install path in README.
3. After one successful binary release, evaluate cargo-binstall metadata.
4. Only consider Homebrew after there is external usage or repeated manual
   install friction.

Rationale: cargo-binstall can fetch binary packages from GitHub Releases when
metadata and artifacts are available, and cargo-dist can automate release
archives and checksums. For this small crate, direct GitHub Release archives are
the lowest-risk first step. They preserve a simple release process and avoid
opening a Homebrew or installer maintenance surface too early.

## Sources Checked

- [`wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [PyO3 user guide](https://pyo3.rs/)
- [maturin user guide](https://www.maturin.rs/)
- [cargo-binstall README](https://github.com/cargo-bins/cargo-binstall)
- [cargo-dist repository](https://github.com/axodotdev/cargo-dist)
