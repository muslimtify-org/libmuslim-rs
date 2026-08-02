# Prayer-times Module Namespace Implementation Plan

> **For executing agents:** implement this plan task-by-task. Each step uses checkbox (`- [ ]`) syntax. Do not skip steps. Do not batch commits across tasks.

**Goal:** Move the safe prayer-times binding under `libmuslim::prayertimes` while retaining `libmuslim-rs` as the Cargo package name.

**Architecture:** `src/lib.rs` becomes a minimal crate root that publicly declares the prayer-times module. The existing safe API moves intact to `src/prayertimes/mod.rs`, while its private C ABI declarations move to `src/prayertimes/ffi.rs`; Cargo's explicit library target supplies the `libmuslim` import name.

**Tech stack:** Rust 2024 edition, Cargo, C FFI, the existing `cc` build dependency, rustfmt, Clippy, rustdoc.

---

## Premortem

**Hidden assumptions:**
- Adding `[lib] name = "libmuslim"` changes Rust import paths without changing the crates.io package name — verify both facts with `cargo metadata --no-deps --format-version 1` and the integration test compiled against `libmuslim`.
- Moving `src/lib.rs` into a module can change how inner attributes and module-relative paths resolve — put `#![warn(missing_docs)]` only in the new crate root and require the complete unit, integration, ABI, and doctest suite to pass after the move.

**Irreversible / risky steps:**
- The namespace change deliberately breaks downstream root-level imports — document it in the README and changelog; each implementation task is isolated in a commit that can be reverted with `git revert`.

**Spec-misalignment:**
- "Change to just libmuslim" could mean renaming the published Cargo package, but the approved design explicitly retains `package.name = "libmuslim-rs"` and changes only the Rust library target name — Task 1 verifies this exact interpretation through Cargo metadata.

**Verify-clause weakness:**
- A general passing test suite would not prove the intended public namespace — Task 1 makes the external-consumer test import the complete safe surface specifically from `libmuslim::prayertimes`, checks a compile-fail doctest for a root-level symbol, and asserts both Cargo target and package names.
- Documentation can become stale without causing ordinary unit-test failures — Task 2 uses repository-wide searches for the old import name and runs doctests plus the updated example.

## File structure

New:
- `src/prayertimes/mod.rs` — the existing safe, idiomatic Rust prayer-times API and its unit tests, moved from the old crate root.
- `src/prayertimes/ffi.rs` — private C-compatible declarations and ABI verification tests, moved from `src/ffi.rs`.

Modified:
- `src/lib.rs` — minimal crate documentation, missing-docs policy, public `prayertimes` declaration, and compile-fail coverage for the removed root API.
- `Cargo.toml` — explicit Rust library target named `libmuslim`; the package name stays unchanged.
- `tests/public_api.rs` — external-consumer coverage through `libmuslim::prayertimes`.
- `examples/basic.rs` — example imports through `libmuslim::prayertimes`.
- `README.md` — installation/import distinction, three-library coverage status, and namespaced usage.
- `CHANGELOG.md` — record the breaking namespace and Rust library-name change.

Removed:
- `src/ffi.rs` — replaced by `src/prayertimes/ffi.rs`.

### Task 1: Establish the `libmuslim::prayertimes` public boundary → verify: the public API integration test imports the complete safe surface from `libmuslim::prayertimes`, the root-level compile-fail doctest passes, and Cargo metadata reports package `libmuslim-rs` with library target `libmuslim`

**Files:**
- Create: `src/prayertimes/mod.rs`
- Create: `src/prayertimes/ffi.rs`
- Modify: `src/lib.rs:1-27`
- Modify: `Cargo.toml:1-39`
- Modify: `tests/public_api.rs:1-7`
- Modify: `examples/basic.rs:1-7`
- Remove: `src/ffi.rs`

- [x] **Step 1: Change the external-consumer test to require the new namespace**

Replace the import at the top of `tests/public_api.rs` with:

```rust
use std::str::FromStr;

use libmuslim::prayertimes::{
    AsrSchool, CalculationMethod, Coordinates, Date, MethodParams, MidnightMode, PrayerTime,
    PrayerTimes, UtcOffset, calculate, constants,
};
```

- [x] **Step 2: Run the public API test and verify the new library name is not available yet**

Run: `cargo test --locked --test public_api`

Expected: the command exits nonzero because the current Cargo library target is still named `libmuslim_rs`, so `libmuslim` cannot be resolved.

- [x] **Step 3: Move the implementation into the prayer-times module**

Run:

```bash
mkdir -p src/prayertimes
mv src/ffi.rs src/prayertimes/ffi.rs
mv src/lib.rs src/prayertimes/mod.rs
```

The move intentionally preserves the complete contents and history of both implementation files. In `src/prayertimes/mod.rs`, remove the crate-only inner attribute:

```rust
#![warn(missing_docs)]
```

Keep `mod ffi;` unchanged; it now resolves to the adjacent `src/prayertimes/ffi.rs`.

- [x] **Step 4: Create the minimal crate root and enforce the breaking boundary**

Create `src/lib.rs` with exactly:

```rust
//! Safe, idiomatic Rust bindings for libmuslim.
//!
//! The currently supported library is available through [`prayertimes`].
//!
//! The old root-level prayer-times API was intentionally removed. Import its
//! items from [`prayertimes`] instead.
//!
//! ```compile_fail
//! use libmuslim::calculate;
//! ```

#![warn(missing_docs)]

/// Astronomical prayer-time calculations backed by `prayertimes.h`.
pub mod prayertimes;
```

Update the doctest at the top of `src/prayertimes/mod.rs` to use the public module and module-owned error:

```rust
//! use libmuslim::prayertimes::{
//!     calculate, CalculationMethod, Coordinates, Date, MethodParams, UtcOffset,
//! };
//!
//! let date = Date::new(2026, 7, 30)?;
//! let coordinates = Coordinates::new(-6.2, 106.8)?;
//! let offset = UtcOffset::from_hours(7.0)?;
//! let mut params = MethodParams::for_method(CalculationMethod::Kemenag)?;
//! params.ihtiyat_minutes = 3;
//!
//! let times = calculate(date, coordinates, offset, &params)?;
//! assert_eq!(times.fajr.format_hm().len(), 5);
//! # Ok::<(), libmuslim::prayertimes::Error>(())
//! ```
```

- [x] **Step 5: Give the Rust library target its public import name**

Add this immediately after the `[package]` metadata in `Cargo.toml`, before `[build-dependencies]`:

```toml
[lib]
name = "libmuslim"
```

Do not alter `package.name = "libmuslim-rs"`.

- [x] **Step 6: Format and verify the new API, ABI, and metadata**

Per the user's Option A resolution after the first dispatch halted, this step also updates the `examples/basic.rs` import to `libmuslim::prayertimes` before running the all-targets test. This keeps the Task 1 commit buildable; the example edit is therefore no longer part of Task 2.

Run: `cargo fmt --all`

Expected: exits zero.

Run: `cargo test --locked --all-targets`

Expected: exits zero, including `external_consumer_can_use_the_complete_safe_surface` and the existing FFI ABI tests.

Run: `cargo test --locked --doc`

Expected: exits zero, including the namespaced usage doctest and the root-level `compile_fail` doctest.

Run: `cargo metadata --no-deps --format-version 1`

Expected: exits zero and its JSON contains a package with `"name":"libmuslim-rs"` whose library target contains `"name":"libmuslim"`.

- [x] **Step 7: Commit the module boundary**

```bash
git add Cargo.toml src/lib.rs src/prayertimes tests/public_api.rs
git add -u src/ffi.rs
git commit -m "refactor: namespace prayer times binding"
```

### Task 2: Update user-facing documentation and run the quality gate → verify: no `libmuslim_rs` import remains; the basic example uses `libmuslim::prayertimes`; formatting, tests, Clippy, rustdoc, and package dry-run all exit zero

**Files:**
- Modify: `README.md:10-65`
- Modify: `CHANGELOG.md:8`

- [ ] **Step 1: Update README coverage and usage**

Replace the introductory coverage list with:

```markdown
Libmuslim-rs provides safe, idiomatic Rust bindings for
[libmuslim](https://github.com/muslimtify-org/libmuslim), a collection of
Muslim-focused C libraries:

1. `prayertimes.h`: astronomical prayer-time calculations with 21 international
   calculation standards (supported)
2. `hijri.h`: astronomical Hijri calendar calculations (not yet supported)
3. `timezone.h`: time-zone support (not yet supported)
```

Replace the package/import explanation with:

```markdown
The Cargo package name is `libmuslim-rs`, while the Rust library import name is
`libmuslim`.
```

Replace the README usage import with:

```rust
use libmuslim::prayertimes::{
    calculate, CalculationMethod, Coordinates, Date, Error, MethodParams, UtcOffset,
};
```

Keep the remainder of the usage example unchanged.

- [ ] **Step 2: Record the breaking change**

Under `## [Unreleased]` in `CHANGELOG.md`, add:

```markdown
### Changed

- **Breaking:** the Rust library is now imported as `libmuslim`, and the
  prayer-times API moved from the crate root to `libmuslim::prayertimes`.
```

- [ ] **Step 3: Verify documentation, example, and repository references**

Run: `rg -n "libmuslim_rs" src tests examples README.md Cargo.toml`

Expected: exits with status 1 and prints no matches.

Run: `cargo test --locked --doc`

Expected: exits zero.

Run: `cargo run --locked --example basic`

Expected: exits zero and prints sections headed `Kemenag preset`, `Adjusted Kemenag preset`, and `Fully custom method`.

Run: `cargo fmt --all --check`

Expected: exits zero.

- [ ] **Step 4: Run the remaining complete quality gate**

Run: `cargo test --locked --all-targets`

Expected: exits zero.

Run: `cargo clippy --locked --all-targets --all-features -- -D warnings`

Expected: exits zero.

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features`

Expected: exits zero.

Run: `cargo publish --locked --dry-run --all-features --allow-dirty`

Expected: exits zero and packages the working-tree version of `libmuslim-rs` successfully; `--allow-dirty` is required because the documentation edits are committed only after this gate passes.

- [ ] **Step 5: Commit the user-facing migration**

```bash
git add README.md CHANGELOG.md examples/basic.rs
git commit -m "docs: update prayertimes namespace usage"
```
