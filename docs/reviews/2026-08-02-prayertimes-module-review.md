# Review — Prayer-times Module Namespace

**Date:** 2026-08-02
**Spec:** `docs/specs/2026-08-02-prayertimes-module-design.md`
**Plan:** `docs/plans/2026-08-02-prayertimes-module.md`
**Verify report:** `docs/verifications/2026-08-02-prayertimes-module-verify.md`
**Commits under review:** `f2b6524..aecde6f` on `vibe/prayertimes-module`

## Diff summary

- Files changed: 10
- Lines added: 1,349; removed: 1,159
- Commits: 5
- Largest construct: `src/prayertimes/mod.rs` at 1,131 lines, moved from the former `src/lib.rs`; it is not a new abstraction or new behavior.
- Simplicity result: Lean already. Net: -0 lines possible without splitting or changing the safe API contrary to the approved design.

## Findings

### Block

- None.

### Warn

- None.

### Nit

- None.

## Self-critique (three risks)

1. Cargo could retain the package name but expose the wrong downstream library name — mitigation: `cargo metadata` reports package `libmuslim-rs` and target `libmuslim`; the external integration test compiles `use libmuslim::prayertimes`.
2. Moving the modules could silently alter a C layout or calculation path — mitigation: all three compiler-backed ABI tests, the Rust/C calculation comparison, and all existing validation and formatting tests pass.
3. A root-level compatibility export could remain even though one compile-fail doctest passes — mitigation: `src/lib.rs` contains only crate documentation, `#![warn(missing_docs)]`, and `pub mod prayertimes`; no safe prayer-time item is re-exported at the root.

## Diff

Review the full verbatim diff with:

```bash
git -C /home/rizukirr/Projects/libmuslim-rs/.vibe-worktrees/2026-08-02-prayertimes-module diff --find-renames f2b6524..aecde6f
```

Changed-file summary:

```text
 CHANGELOG.md                                       |    5 +
 Cargo.toml                                         |    3 +
 README.md                                          |   15 +-
 docs/plans/2026-08-02-prayertimes-module.md        |   41 +-
 .../2026-08-02-prayertimes-module-verify.md        |  174 +++
 examples/basic.rs                                  |    2 +-
 src/lib.rs                                         | 1135 +-------------------
 src/{ => prayertimes}/ffi.rs                       |    0
 src/prayertimes/mod.rs                             | 1131 +++++++++++++++++++
 tests/public_api.rs                                |    2 +-
 10 files changed, 1349 insertions(+), 1159 deletions(-)
```

## Sign-off

- [ ] User reviewed findings.
- [ ] User reviewed diff.
- [ ] User approves proceeding to finish-branch.
