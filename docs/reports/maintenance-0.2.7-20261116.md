# 0.2.7 maintenance readiness — 2026-11-16

## Decision and stop conditions

Prepare 0.2.7 as an **unpublished maintenance candidate**. Do not publish, tag,
upgrade downstream users or add parser features in this work.

The release decision needs evidence that the two fixes since 0.2.6 are covered,
the unchanged public API/content policies pass existing regressions, Rust 1.70
still builds the dependency-free core and CLI, and the generated crate can be
verified and dry-run published. Stop the release recommendation if this requires
an API change, a default content-policy change or unrelated parser work.

## Provenance and changes

- Published comparison: tag `v0.2.6`, crate 0.2.6.
- Reviewed main source: `41dfbb3c93ba4351fbd3048e7e03fc453c4c8253`.
- Runtime differences: `3f6d645` consumes escaped CRLF as one continuation;
  `41dfbb3` preserves native path units after CLI `--output=`.
- Existing regression additions cover quoted keys/values, LF/CR/CRLF, adjacent
  literal content, all four content-option combinations, strict mode, chunked
  readers, Unix byte parsing and Windows UTF-16 parsing. No duplicate parser
  implementation or new test harness is added to the repository.
- This candidate changes version metadata/locks, changelog and release documents
  only. README clearly identifies 0.2.7 as unpublished and retains working 0.2.6
  install examples. The fuzz lock records the same 0.2.7 path dependency.

## Local evidence

Environment: macOS arm64; stable validation explicitly uses Rust/Cargo 1.99.1.
MSRV uses the separately installed Rust/Cargo 1.70.0. Raw command outputs are in
`target/release-readiness-20261116/` in this isolated worktree; they are not
included in the crate. Package hashes and the final source revision are recorded
there separately, so this document does not contain a self-referential package
hash.

| Check | Evidence / status |
| --- | --- |
| Deny-warning all-target check | Passed; `01-check.log` |
| Formatting | Passed; `02-fmt.log` |
| Strict all-feature/all-target Clippy | Passed; `03-clippy.log` |
| Default and all-feature tests | Passed: 333 default and 350 all-feature tests, zero failures/ignores; `04-default-tests.log` and `05-all-feature-tests.log` |
| All-feature doctests and rustdoc | Passed: one doctest and generated library/CLI rustdoc; `06-doctests.log` and `07-doc.log` |
| Rust 1.70 core + CLI | Passed with `--lib --bins --no-default-features --locked`; `08-msrv.log` and `08-msrv-offline.log` |
| 0.2.6/candidate behavior comparison | Passed; `09-baseline-continuation.log` |
| Fuzz path dependency metadata | `10-fuzz-metadata.json`; no fuzz run claimed |
| Package/list/verification and publish dry run | Passed: package verified and upload explicitly aborted by dry-run; `11-package.log`, `12-package-list.log`, `13-publish-dry-run.log` |
| Package contents and packaged MSRV | Passed: 82 files, required source/tests/docs/license present, no target/.omx/.git artifacts; lock v3; packaged core/CLI pass Rust 1.70 offline; `14-package-inspection.json`, `15-packaged-msrv.log` |

Initial package verification used `--allow-dirty`, as permitted by the checklist
for an unreleased branch. After the reviewed candidate is committed, clean-source
package/list/dry-run outputs and final artifact/source hashes are recorded in the
same local evidence directory. No tagged or public package is implied.

The MSRV's first online check encountered crates.io index timeouts before it
completed successfully; the offline recheck also passed. The core build did not
require a dependency downgrade or lockfile rewrite.

For the behavior comparison, the 0.2.6 library was extracted from its Git tag and
both libraries were compiled directly with Rust 1.70.0. A synthetic quoted value
containing an escaped CRLF repairs to a string with a leftover newline in 0.2.6,
and to `helloworld` in the candidate. Ordinary malformed-object repair, LF and CR
continuations, and a quoted Windows-style path agree across both versions. This
is a narrow library comparison, not a downstream application test.

## Remaining acceptance boundaries

Native Windows UTF-16 and Linux raw-filename execution must be supported by
separate platform evidence; macOS/APFS does not execute the Linux raw-filename
fixture. The repository already has Linux stable/MSRV and Windows content/CLI
jobs, but adding a CI job is not a passing run. Baseline CI could not be read back:
GitHub CLI/API returned a TLS handshake timeout; the one bounded independent API
retry returned `RemoteDisconnected`. Thus neither baseline nor candidate Linux
or Windows success is claimed here. A candidate PR must obtain its own CI runs.

No production payloads, real downstream workspace, Python/Node binding, new
incremental parser, performance improvement, fuzz campaign or full schema
validation was tested or added. The existing memory report is not rerun.
Publication, a new release tag/GitHub Release, docs.rs readback and downstream
acceptance remain separate owner-authorized steps.

已验：Rust 1.70 对照中看到 CRLF 残留换行被修正，且相邻 LF/CR/普通对象/路径一致。
没验：本机未运行原生 Windows 与 Linux 文件系统场景，未发布 0.2.7。
相邻情况：严格模式、四种内容策略与 reader/writer 边界的既有回归已通过；Windows UTF-16 与 Linux 原始文件名仍待候选 CI。
