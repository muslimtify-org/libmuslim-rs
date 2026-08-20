# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Synced the vendored `timezone.h` to libmuslim `cdbcc22`, which enumerates the Windows timezone registry once for the whole table rather than on every call to `parse_timezone_offset`. That lookup cost 41 ms on Windows against roughly 6 microseconds on Linux, and a Windows runner now measures 0.0255 ms ([libmuslim#64](https://github.com/muslimtify-org/libmuslim/issues/64)).

- `concurrent_lookups_of_different_zones_stay_correct` is back to 50000 iterations on every platform. It had been cut to 500 on Windows in #14 purely to stop CI spending 68 minutes on the upstream defect, and that workaround is no longer needed.

### Fixed

- Synced the vendored `prayertimes.h` to `v0.2.1`, libmuslim release `2026.08.20`, which carries three high-latitude fixes. Prayers no longer come back out of order inside the polar circle, where the previous release reported isha before maghrib on 41 days a year at Longyearbyen under MWL ([libmuslim#68](https://github.com/muslimtify-org/libmuslim/pull/68)). asr is no longer reported where the Sun casts no shadow, which had returned a formula artifact as a prayer time at 2926 points of a 40810 point grid ([libmuslim#69](https://github.com/muslimtify-org/libmuslim/pull/69)). The substituted isha is anchored on maghrib rather than sunset, removing the last case where it could precede maghrib ([libmuslim#70](https://github.com/muslimtify-org/libmuslim/pull/70)).

### Changed

- **Behaviour.** `calculate` now returns `Error::NonFiniteResult("asr")` on a narrow band of days where it previously returned times. At Longyearbyen that is four days a year, where the separation from the declination sits between 90 and 90.833 degrees: the Sun is visible only by refraction, so sunrise exists and fajr, maghrib and isha all resolve, but no shadow is cast and asr does not occur. Previously the C library returned an artifact there and this crate passed it through as a real time.

  This crate fails the whole calculation when any field is non-finite, so four valid times are withheld along with the impossible one. That is the same shape as the dhuha problem libmuslim#63 solved upstream by removing the field, and it is pinned by a test rather than left to chance. Whether `asr` should become an `Option` is open.

### Added

- `HighLatMethod` and two `MethodParams` fields, `high_lat_method` and
  `high_lat_reference_latitude`, surfacing libmuslim v0.2.0's per-method
  high-latitude rule. The `ffi` mirror of this enum had been kept unexported
  against exactly this change, so its discriminants were already pinned
  ([libmuslim#62](https://github.com/muslimtify-org/libmuslim/pull/62)).

### Changed

- **Breaking.** `PrayerTimes` carries the five prescribed prayers only. `sunrise` and `dhuha` are removed, along with `constants::DHUHA_ALTITUDE`. Sunrise is the end of the fajr window rather than a prayer, and dhuha is a voluntary prayer carried only by Indonesian timetables. Both are still computed inside the C library, because maghrib is sunset and every high-latitude substitution measures the night between sunset and sunrise, but neither is part of the contract ([libmuslim#63](https://github.com/muslimtify-org/libmuslim/pull/63)).

  This replaces the entry that stood here after #11, which made `dhuha` an `Option<PrayerTime>`. That was this crate's answer to a non-finite dhuha withholding the five prescribed times with `Error::NonFiniteResult`. Removing the field upstream answers the same problem at the source, so the `Option` reached no release and its entry is gone rather than superseded in place. Callers who read `times.sunrise` have no replacement in this crate.

### Fixed

- `format_hm` and `format_hms` could return a string with a negative field, and
  the `hour`, `minute` and `second` accessors could return negative components,
  for any time outside 0 to 24 hours. `calculate` returns such values at high
  latitude, so this was reachable rather than defensive. Both the vendored
  `prayertimes.h` and the Rust reimplementation in `hms_components` now reduce
  onto the clock face first, and a test asserts the two agree
  ([libmuslim#57](https://github.com/muslimtify-org/libmuslim/pull/57)).
  Before this, a decimal hour of -0.104 formatted as `00:-6:-13` and
  decomposed to `(0, -6, -13)`.

- Prayer times were computed from a solar position evaluated once at 0h UT and
  reused for events up to 20 hours later, so the declination was stale by the
  time sunset was solved. The vendored `prayertimes.h` now evaluates the Sun at
  each event's own instant
  ([libmuslim#49](https://github.com/muslimtify-org/libmuslim/pull/49)).
  Maghrib is measured against a JPL DE440 validated solver at 6.5966 seconds
  worst case over a grid of 14235 points, for `|latitude| <= 60`.

### Changed

- Computed times move at high latitudes. Measured across 12 cities and all of
  2026, 24192 values, the worst movement is 0 minutes for dhuhr and asr,
  1 for sunrise, 2 for fajr, 3 for maghrib and 16 for isha. Equatorial results
  move by at most 1 minute. The isha figure is Stockholm on 2026-08-17, where
  the high latitude fallback engages and is steep, so a small input shift
  produces a large output shift. Callers above roughly 55 degrees should read
  this as a correction rather than drift, but note that the isha refinement has
  no oracle behind it upstream
  ([libmuslim#52](https://github.com/muslimtify-org/libmuslim/issues/52)).

### Known issues

- `PrayerTimes::dhuha` is NAN above roughly 62.5 degrees of latitude on the
  days when the Sun never reaches the dhuha altitude, with no error and no
  sentinel. At Reykjavik that is 40 days a year. Sunrise and dhuhr solve
  normally on those same days, so only this one field is affected
  ([libmuslim#51](https://github.com/muslimtify-org/libmuslim/issues/51)).

## [0.3.0] - 2026-08-16

### Fixed

- `CalculationMethod::from_str` rejected `"CUSTOM"` and `"Custom"` while
  accepting every other method name in any casing. The C lookup folds case,
  but the Rust guard that separates a genuine `Custom` from the C not-found
  sentinel compared case-sensitively.
- `timezone::offset_at` returned `Ok(0.0)` for a zone the host cannot resolve,
  making a typo such as `"Asia/Jakata"` indistinguishable from real UTC and
  silently shifting every calculated prayer time. Zone names are now checked
  against the host zone database first.
- `timezone::offset_at` raced with any other thread in the host process that
  read `TZ`, called `localtime` or called `tzset`. The vendored `timezone.h`
  resolved offsets by mutating the process environment; it now reads the
  zone's TZif file directly and mutates nothing global
  ([libmuslim#41](https://github.com/muslimtify-org/libmuslim/issues/41)).
  The crate's mutex, which only ever covered its own callers, is gone.

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

[Unreleased]: https://github.com/muslimtify-org/libmuslim-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/muslimtify-org/libmuslim-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/muslimtify-org/libmuslim-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/muslimtify-org/libmuslim-rs/releases/tag/v0.1.0
