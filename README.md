## Libmuslim-rs

[![CI](https://github.com/muslimtify-org/libmuslim-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/muslimtify-org/libmuslim-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/libmuslim-rs.svg)](https://crates.io/crates/libmuslim-rs)
[![docs.rs](https://docs.rs/libmuslim-rs/badge.svg)](https://docs.rs/libmuslim-rs)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)](https://releases.rs)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Libmuslim-rs provides safe, idiomatic Rust bindings for
[libmuslim](https://github.com/muslimtify-org/libmuslim), a collection of
Muslim-focused C libraries:

1. `prayertimes.h`: astronomical prayer-time calculations with 21 international
   calculation standards (supported)
2. `hijri.h`: astronomical Hijri calendar calculations (not yet supported)
3. `timezone.h`: host-backed IANA time-zone and DST support (supported)

for more information, about usage and API documentation, please visit [muslimtify](https://muslimtify.vercel.app/docs/category/rust) documentation website

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libmuslim-rs = "0.2.0"
```

or run this command in your terminal:

```bash
cargo add libmuslim-rs
```

The Cargo package name is `libmuslim-rs`, while the Rust library import name is
`libmuslim`.

### Requirements

- Rust 1.85 or newer (the crate uses edition 2024).
- A C11-capable C compiler on the build host. The C sources are vendored in
  `include/` and compiled by `build.rs` via [`cc`](https://crates.io/crates/cc),
  so there is no system library to install — but `gcc`, `clang`, or MSVC must be
  available. CI builds on Linux, macOS, and Windows.

## Usage

```rust
use libmuslim::prayertimes::{
    calculate, CalculationMethod, Coordinates, Date, MethodParams,
};
use libmuslim::timezone::offset_at;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let date = Date::new(2026, 7, 30)?;
    let coordinates = Coordinates::new(-6.2, 106.8)?;

    // Resolve the offset through the host timezone database. A fixed
    // UtcOffset::from_hours(7.0)? can still be supplied instead.
    let offset = offset_at("Asia/Jakarta", 1_785_369_600)?;

    let mut params = MethodParams::for_method(CalculationMethod::Kemenag)?;
    params.ihtiyat_minutes = 3;

    let times = calculate(date, coordinates, offset, &params)?;
    println!("Fajr: {}", times.fajr.format_hm());

    Ok(())
}
```

### Resolve the offset at the current time

This is the Rust equivalent of calling C's `time(NULL)` and passing the result
through `parse_timezone_offset()`:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

use libmuslim::prayertimes::{
    CalculationMethod, Coordinates, Date, MethodParams, calculate,
};
use libmuslim::timezone::offset_at;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let when = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())?;
    let offset = offset_at("Asia/Jakarta", when)?;

    let params = MethodParams::for_method(CalculationMethod::Kemenag)?;
    let times = calculate(
        Date::new(2026, 7, 12)?,
        Coordinates::new(-6.2088, 106.8456)?,
        offset,
        &params,
    )?;

    println!("Dhuhr {}", times.dhuhr.format_hm());
    Ok(())
}
```

The `timezone` module also provides `system_timezone()` for detecting the host
zone. It relies on the operating system's timezone database (or Windows timezone
APIs). The upstream C API returns `0.0` both for a real UTC offset and for an
unresolved zone, so `offset_at` preserves that ambiguity rather than claiming
it can distinguish the two cases.

For a fuller walkthrough, including a fully custom calculation method, see
[`examples/basic.rs`](examples/basic.rs):

```bash
cargo run --example basic
```

## Development

```bash
cargo test --locked --all-targets   # unit, integration, and ABI-layout tests
cargo test --locked --doc           # documentation examples
cargo fmt --all --check             # formatting
cargo clippy --locked --all-targets -- -D warnings
```

`Cargo.lock` is committed and CI builds with `--locked`, so dependency changes
land as explicit lockfile commits (Dependabot opens these monthly) rather than
appearing silently in an unrelated pull request.

CI runs all of the above on every pull request, plus a rustdoc build with
warnings denied, an MSRV check, `cargo deny` for advisories and licenses, a
`cargo publish --dry-run` packaging check, and the test suite under
UndefinedBehaviorSanitizer to exercise the vendored C code.

## Releasing

Tagging `v*` builds per-platform artifacts and attaches them to a GitHub
Release. Publishing to crates.io stays manual — see
[RELEASING.md](RELEASING.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
