---
title: Prayer-times module namespace
date: 2026-08-02
status: draft
---

# Prayer-times Module Namespace — Design

## Problem
The Rust binding currently exposes prayer-time functionality directly from the crate root, even though the upstream libmuslim project consists of three libraries: `prayertimes.h`, `hijri.h`, and `timezone.h`. The binding currently covers only `prayertimes.h`, and its structure does not provide a clear namespace or repeatable layout for adding the other libraries later.

## Goals
- Expose the existing safe API under `libmuslim::prayertimes`.
- Keep the Cargo package name `libmuslim-rs` while making the Rust library name `libmuslim`.
- Separate the safe prayer-times API from its private unsafe C ABI declarations.
- Establish a module pattern that future `hijri` and `timezone` bindings can follow.
- Update README documentation, doctests, examples, and tests for the new namespace.
- Preserve all existing calculation, validation, formatting, error, and ABI behavior.

## Non-goals
- Preserve the old root-level Rust API or provide deprecated compatibility re-exports.
- Add bindings for `hijri.h` or `timezone.h`.
- Change the vendored C implementation or prayer-time calculation behavior.
- Split the safe prayer-times API into additional files beyond `mod.rs` during this refactor.

## Constraints
- This is an intentional breaking API change.
- The crates.io package remains named `libmuslim-rs`.
- The compiled Rust library is named `libmuslim`, so downstream imports start with `libmuslim`.
- Raw FFI declarations must remain private and unsafe operations must stay confined to the prayer-times implementation.
- Existing C ABI layout and discriminant tests must continue to pass.

## Approach
Keep `src/lib.rs` as a minimal crate root containing crate-level documentation and `pub mod prayertimes`. Move the current safe Rust API from `src/lib.rs` to `src/prayertimes/mod.rs`, including public types, constants, validation, formatting, calculation wrappers, and the module-owned `prayertimes::Error`. Move the existing raw bindings from `src/ffi.rs` to the private module `src/prayertimes/ffi.rs`.

Add a `[lib]` section to `Cargo.toml` with `name = "libmuslim"`, without changing `package.name`. Public consumers will use imports such as:

```rust
use libmuslim::prayertimes::{
    calculate, CalculationMethod, Coordinates, Date, MethodParams, UtcOffset,
};
```

A calculation continues to flow through the safe validation and conversion layer in `prayertimes/mod.rs`, then through private declarations in `prayertimes/ffi.rs`, and finally into the existing vendored C implementation. No root-level compatibility re-exports will be added.

Update `README.md`, crate and item doctests, `examples/basic.rs`, and integration tests to use the new library and module paths. The native build remains limited to the existing prayer-times C source.

## Alternatives considered
One alternative was a more granular prayer-times module with separate `error.rs`, `types.rs`, `constants.rs`, and `ffi.rs` files. This could help after the API grows, but adds navigation and re-export overhead without a current need.

Another alternative was a minimal namespace shim that retained implementation at the crate root and re-exported it from `prayertimes`. Although it would require fewer changes, it would preserve the architectural mismatch and provide a poor pattern for future `hijri` and `timezone` modules.

## Testing
- Run the formatter and compile the crate.
- Run all unit, ABI, integration, and documentation tests.
- Update public API tests to compile against `libmuslim::prayertimes`.
- Verify README code, crate doctests, and `examples/basic.rs` use the new namespace.
- Verify the old root-level symbols are not re-exported.
- Verify the Cargo package remains `libmuslim-rs` while downstream Rust imports use `libmuslim`.

## Open questions
N/A — the namespace, compatibility policy, ownership boundaries, and scope were agreed during design review.
