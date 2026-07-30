# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/muslimtify-org/libmuslim-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/muslimtify-org/libmuslim-rs/releases/tag/v0.1.0
