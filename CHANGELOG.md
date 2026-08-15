# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- `CalculationMethod::from_str` rejected `"CUSTOM"` and `"Custom"` while
  accepting every other method name in any casing. The C lookup folds case,
  but the Rust guard that separates a genuine `Custom` from the C not-found
  sentinel compared case-sensitively.
- `timezone::offset_at` returned `Ok(0.0)` for a zone the host cannot resolve,
  making a typo such as `"Asia/Jakata"` indistinguishable from real UTC and
  silently shifting every calculated prayer time. Zone names are now checked
  against the host zone database first.

### Changed

- **Breaking:** `timezone::offset_at` returns the new
  `TimezoneError::UnknownZone` for an unresolvable zone instead of falling
  back to a `0.0` offset. Only IANA zone names present in the host database
  are accepted; bare POSIX TZ strings such as `"XYZ8"` are rejected on every
  platform.

### Added

- `abi_probe.c` pins the public `prayertimes.h` prototypes, so a changed C
  signature fails the build instead of silently diverging from the
  hand-written `extern "C"` declarations in `prayertimes::ffi`.

## [0.2.0] - 2026-08-02

### Added

- Safe `timezone` bindings for resolving DST-aware IANA UTC offsets and
  detecting the host system time zone through the vendored `timezone.h`.

### Changed

- **Breaking:** the Rust library is now imported as `libmuslim`, and the
  prayer-times API moved from the crate root to `libmuslim::prayertimes`.

## [0.1.0] - 2026-07-30

Initial release.

### Added

- Safe Rust bindings over the vendored `prayertimes.h` C implementation.
- `calculate` entry point returning the seven daily prayer times (Fajr,
  Sunrise, Dhuha, Dhuhr, Asr, Maghrib, Isha).
- Validated newtypes for the inputs the C layer would otherwise accept
  silently: `Date`, `Coordinates`, `UtcOffset`, and `PrayerTime`.
- `CalculationMethod` covering 21 international standards plus `Custom`, with
  `MethodParams::for_method` presets and `FromStr` / `as_str` round-tripping.
- `AsrSchool` and `MidnightMode` configuration enums.
- `constants` module mirroring the astronomical constants in the C header, with
  tests asserting the Rust and C values agree.
- ABI layout tests that compare Rust struct sizes, alignments, and field
  offsets against the C compiler's own `sizeof` / `offsetof` / `_Alignof`.

### Notes

- No high-latitude strategy selector is exposed. The C header declares a
  `HighLatMethod` enum, but it is not a field of `MethodParams` and the
  calculation never reads it — the fallback is hardcoded to the angle-based
  rule, and `MIDDLE_OF_NIGHT` / `ONE_SEVENTH` are unimplemented upstream.
  Publishing a selector that cannot affect a result would be misleading.
- `MethodParams::midnight_mode` crosses the FFI boundary faithfully but is not
  read by the C calculation, and `Standard` is its only value. No midnight or
  qiyam time is returned, because `struct PrayerTimes` has no field for one.

[Unreleased]: https://github.com/muslimtify-org/libmuslim-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/muslimtify-org/libmuslim-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/muslimtify-org/libmuslim-rs/releases/tag/v0.1.0
