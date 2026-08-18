//! Safe Rust prayer-time calculations backed by libprayertime.
//!
//! The underlying C library expects an explicit UTC offset for each
//! calculation. It does not look up time zones or apply daylight-saving
//! rules, so callers must supply the offset that applies to the requested date
//! and location.
//!
//! # Example
//!
//! ```
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

mod ffi;

use std::ffi::{CStr, CString, c_char};
use std::str::FromStr;

/// Mathematical and astronomical constants used by the prayer-time algorithm.
pub mod constants {
    /// Multiplier for converting degrees to radians.
    pub const DEGREES_TO_RADIANS: f64 = std::f64::consts::PI / 180.0;
    /// Multiplier for converting radians to degrees.
    pub const RADIANS_TO_DEGREES: f64 = 180.0 / std::f64::consts::PI;
    /// Julian date of the J2000 epoch.
    pub const JULIAN_EPOCH: f64 = 2_451_545.0;
    /// Sun mean anomaly offset in degrees.
    pub const SUN_MEAN_ANOMALY_OFFSET: f64 = 357.529;
    /// Sun mean anomaly daily rate in degrees.
    pub const SUN_MEAN_ANOMALY_RATE: f64 = 0.98560028;
    /// Sun mean longitude offset in degrees.
    pub const SUN_MEAN_LONGITUDE_OFFSET: f64 = 280.459;
    /// Sun mean longitude daily rate in degrees.
    pub const SUN_MEAN_LONGITUDE_RATE: f64 = 0.98564736;
    /// First solar equation-of-center amplitude.
    pub const SUN_ECCENTRICITY_AMPLITUDE_1: f64 = 1.915;
    /// Second solar equation-of-center amplitude.
    pub const SUN_ECCENTRICITY_AMPLITUDE_2: f64 = 0.020;
    /// Earth's axial obliquity coefficient in degrees.
    pub const OBLIQUITY_COEFFICIENT: f64 = 23.439;
    /// Earth's axial obliquity daily rate in degrees.
    pub const OBLIQUITY_RATE: f64 = 0.00000036;
    /// Atmospheric refraction correction in degrees.
    pub const REFRACTION_CORRECTION: f64 = 0.833;
    /// Solar altitude used for Dhuha in degrees.
    pub const DHUHA_ALTITUDE: f64 = 4.3;
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
/// An error produced while validating input or calling libprayertime.
pub enum Error {
    /// The supplied Gregorian calendar date is invalid.
    InvalidDate {
        /// The invalid year.
        year: i32,
        /// The invalid month.
        month: u8,
        /// The invalid day of the month.
        day: u8,
    },
    /// The latitude, longitude, or both are non-finite or out of range.
    InvalidCoordinates {
        /// The invalid latitude.
        latitude: f64,
        /// The invalid longitude.
        longitude: f64,
    },
    /// The UTC offset is non-finite or outside the supported range.
    InvalidUtcOffset {
        /// The invalid offset in hours.
        hours: f64,
    },
    /// A calculation-method parameter is invalid.
    InvalidMethodParams {
        /// The invalid parameter's name.
        field: &'static str,
        /// A description of the required value.
        reason: &'static str,
    },
    /// A calculation-method name was not recognized.
    UnknownCalculationMethod(String),
    /// A string passed to C contains an interior NUL byte.
    StringContainsNul,
    /// A string returned by C is not valid UTF-8.
    InvalidCString,
    /// A C function unexpectedly returned a null pointer.
    NullPointer(&'static str),
    /// A calculated prayer time is not finite.
    NonFiniteResult(&'static str),
    /// A Unix-epoch day cannot be represented by [`Date`].
    DateOutOfRange(i64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDate { year, month, day } => {
                write!(formatter, "invalid date: {year:04}-{month:02}-{day:02}")
            }
            Self::InvalidCoordinates {
                latitude,
                longitude,
            } => write!(
                formatter,
                "invalid coordinates: latitude {latitude}, longitude {longitude}"
            ),
            Self::InvalidUtcOffset { hours } => {
                write!(formatter, "invalid UTC offset: {hours} hours")
            }
            Self::InvalidMethodParams { field, reason } => {
                write!(formatter, "invalid method parameter `{field}`: {reason}")
            }
            Self::UnknownCalculationMethod(method) => {
                write!(formatter, "unknown calculation method: {method}")
            }
            Self::StringContainsNul => write!(formatter, "string contains an interior NUL byte"),
            Self::InvalidCString => write!(formatter, "C string is not valid UTF-8"),
            Self::NullPointer(name) => write!(formatter, "C function returned null for {name}"),
            Self::NonFiniteResult(name) => {
                write!(formatter, "calculation returned a non-finite {name} value")
            }
            Self::DateOutOfRange(days) => {
                write!(formatter, "date for Unix epoch day {days} is out of range")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A built-in prayer-time calculation method.
pub enum CalculationMethod {
    /// Muslim World League.
    Mwl,
    /// Umm al-Qura University, Makkah.
    Makkah,
    /// Islamic Society of North America.
    Isna,
    /// Egyptian General Authority of Survey.
    Egypt,
    /// University of Islamic Sciences, Karachi.
    Karachi,
    /// Türkiye Presidency of Religious Affairs.
    Turkey,
    /// Majlis Ugama Islam Singapura.
    Singapore,
    /// Department of Islamic Development Malaysia.
    Jakim,
    /// Ministry of Religious Affairs of Indonesia.
    Kemenag,
    /// Union of Islamic Organisations of France.
    France,
    /// Spiritual Administration of Muslims of Russia.
    Russia,
    /// Dubai method.
    Dubai,
    /// Qatar method.
    Qatar,
    /// Kuwait method.
    Kuwait,
    /// Jordan method.
    Jordan,
    /// Gulf region method.
    Gulf,
    /// Tunisia method.
    Tunisia,
    /// Algeria method.
    Algeria,
    /// Morocco method.
    Morocco,
    /// Portugal method.
    Portugal,
    /// Moonsighting Committee method.
    Moonsighting,
    /// Caller-supplied custom parameters.
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The juristic shadow-length rule used to calculate Asr.
pub enum AsrSchool {
    /// Standard one-shadow-length rule.
    Standard,
    /// Hanafi two-shadow-length rule.
    Hanafi,
}

// The C header declares a `HighLatMethod` enum, but it is not a field of
// `MethodParams` and `calculate_prayer_times` never reads it: the high-latitude
// fallback is hardcoded to the angle-based rule. No selector is exposed here
// rather than publishing an enum that cannot affect a calculation. The
// discriminants are still asserted against C in `ffi`, so a future upstream
// release that wires this up can be surfaced without re-deriving them.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The convention used to determine midnight.
///
/// Retained because it is a real field of the C `MethodParams` struct, but the
/// C calculation does not currently read it, and no midnight time is returned.
/// `Standard` is the only value the C header defines.
pub enum MidnightMode {
    /// Use the standard sunset-to-sunrise midpoint.
    Standard,
}

impl From<CalculationMethod> for ffi::CalcMethod {
    fn from(method: CalculationMethod) -> Self {
        match method {
            CalculationMethod::Mwl => Self::Mwl,
            CalculationMethod::Makkah => Self::Makkah,
            CalculationMethod::Isna => Self::Isna,
            CalculationMethod::Egypt => Self::Egypt,
            CalculationMethod::Karachi => Self::Karachi,
            CalculationMethod::Turkey => Self::Turkey,
            CalculationMethod::Singapore => Self::Singapore,
            CalculationMethod::Jakim => Self::Jakim,
            CalculationMethod::Kemenag => Self::Kemenag,
            CalculationMethod::France => Self::France,
            CalculationMethod::Russia => Self::Russia,
            CalculationMethod::Dubai => Self::Dubai,
            CalculationMethod::Qatar => Self::Qatar,
            CalculationMethod::Kuwait => Self::Kuwait,
            CalculationMethod::Jordan => Self::Jordan,
            CalculationMethod::Gulf => Self::Gulf,
            CalculationMethod::Tunisia => Self::Tunisia,
            CalculationMethod::Algeria => Self::Algeria,
            CalculationMethod::Morocco => Self::Morocco,
            CalculationMethod::Portugal => Self::Portugal,
            CalculationMethod::Moonsighting => Self::Moonsighting,
            CalculationMethod::Custom => Self::Custom,
        }
    }
}

impl From<ffi::CalcMethod> for CalculationMethod {
    fn from(method: ffi::CalcMethod) -> Self {
        match method {
            ffi::CalcMethod::Mwl => Self::Mwl,
            ffi::CalcMethod::Makkah => Self::Makkah,
            ffi::CalcMethod::Isna => Self::Isna,
            ffi::CalcMethod::Egypt => Self::Egypt,
            ffi::CalcMethod::Karachi => Self::Karachi,
            ffi::CalcMethod::Turkey => Self::Turkey,
            ffi::CalcMethod::Singapore => Self::Singapore,
            ffi::CalcMethod::Jakim => Self::Jakim,
            ffi::CalcMethod::Kemenag => Self::Kemenag,
            ffi::CalcMethod::France => Self::France,
            ffi::CalcMethod::Russia => Self::Russia,
            ffi::CalcMethod::Dubai => Self::Dubai,
            ffi::CalcMethod::Qatar => Self::Qatar,
            ffi::CalcMethod::Kuwait => Self::Kuwait,
            ffi::CalcMethod::Jordan => Self::Jordan,
            ffi::CalcMethod::Gulf => Self::Gulf,
            ffi::CalcMethod::Tunisia => Self::Tunisia,
            ffi::CalcMethod::Algeria => Self::Algeria,
            ffi::CalcMethod::Morocco => Self::Morocco,
            ffi::CalcMethod::Portugal => Self::Portugal,
            ffi::CalcMethod::Moonsighting => Self::Moonsighting,
            ffi::CalcMethod::Custom => Self::Custom,
            ffi::CalcMethod::Count => unreachable!("CALC_COUNT is not a calculation method"),
        }
    }
}

impl From<AsrSchool> for ffi::AsrSchool {
    fn from(school: AsrSchool) -> Self {
        match school {
            AsrSchool::Standard => Self::Standard,
            AsrSchool::Hanafi => Self::Hanafi,
        }
    }
}

impl From<ffi::AsrSchool> for AsrSchool {
    fn from(school: ffi::AsrSchool) -> Self {
        match school {
            ffi::AsrSchool::Standard => Self::Standard,
            ffi::AsrSchool::Hanafi => Self::Hanafi,
        }
    }
}

impl From<MidnightMode> for ffi::MidnightMode {
    fn from(mode: MidnightMode) -> Self {
        match mode {
            MidnightMode::Standard => Self::Standard,
        }
    }
}

impl From<ffi::MidnightMode> for MidnightMode {
    fn from(mode: ffi::MidnightMode) -> Self {
        match mode {
            ffi::MidnightMode::Standard => Self::Standard,
        }
    }
}

impl FromStr for CalculationMethod {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let name = CString::new(input).map_err(|_| Error::StringContainsNul)?;
        let raw = unsafe { ffi::method_from_string(name.as_ptr()) };
        // C returns CALC_CUSTOM both for a real "custom" and for anything it
        // does not recognise, so the only way to tell them apart is to re-test
        // the input here. The comparison must fold case the same way C's
        // pt__ascii_casecmp does -- ASCII-only -- or "CUSTOM" parses as an
        // unknown method while "MWL" parses fine.
        if raw == ffi::CalcMethod::Custom && !input.eq_ignore_ascii_case("custom") {
            return Err(Error::UnknownCalculationMethod(input.to_owned()));
        }
        Ok(raw.into())
    }
}

impl CalculationMethod {
    /// Returns the canonical lowercase name understood by libprayertime.
    pub fn as_str(self) -> &'static str {
        let name = unsafe { ffi::method_to_string(self.into()) };
        assert!(!name.is_null(), "method_to_string returned null");
        unsafe { CStr::from_ptr(name) }
            .to_str()
            .expect("method_to_string returned non-UTF-8")
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Parameters controlling a prayer-time calculation.
pub struct MethodParams {
    /// Human-readable method name passed to libprayertime.
    pub name: String,
    /// Solar depression angle for Fajr, in degrees.
    pub fajr_angle: f64,
    /// Solar depression angle for Isha, in degrees.
    pub isha_angle: f64,
    /// Fixed Isha interval after Maghrib, in minutes; zero uses the angle.
    pub isha_interval_minutes: i32,
    /// Fixed Maghrib interval after sunset, in minutes.
    pub maghrib_interval_minutes: i32,
    /// Juristic rule used for Asr.
    pub asr_school: AsrSchool,
    /// Convention used to determine midnight.
    pub midnight_mode: MidnightMode,
    /// Precautionary adjustment added to calculated times, in minutes.
    pub ihtiyat_minutes: i32,
}

impl MethodParams {
    /// Creates zeroed custom parameters with the given display name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fajr_angle: 0.0,
            isha_angle: 0.0,
            isha_interval_minutes: 0,
            maghrib_interval_minutes: 0,
            asr_school: AsrSchool::Standard,
            midnight_mode: MidnightMode::Standard,
            ihtiyat_minutes: 0,
        }
    }

    /// Loads the parameters for a built-in calculation method.
    pub fn for_method(method: CalculationMethod) -> Result<Self, Error> {
        let params = unsafe { ffi::method_params_get(method.into()) };
        if params.is_null() {
            return Err(Error::NullPointer("method_params_get"));
        }
        let params = unsafe { &*params };
        if params.name.is_null() {
            return Err(Error::NullPointer("method parameter name"));
        }
        let name = unsafe { CStr::from_ptr(params.name) }
            .to_str()
            .map_err(|_| Error::InvalidCString)?
            .to_owned();
        let asr_school = match params.asr_shadow {
            1 => AsrSchool::Standard,
            2 => AsrSchool::Hanafi,
            _ => {
                return Err(Error::InvalidMethodParams {
                    field: "asr_school",
                    reason: "C preset has an invalid shadow factor",
                });
            }
        };
        Ok(Self {
            name,
            fajr_angle: params.fajr_angle,
            isha_angle: params.isha_angle,
            isha_interval_minutes: params.isha_interval,
            maghrib_interval_minutes: params.maghrib_interval,
            asr_school,
            midnight_mode: params.midnight_mode.into(),
            ihtiyat_minutes: params.ihtiyat,
        })
    }

    /// Validates that the parameters can be passed safely to libprayertime.
    pub fn validate(&self) -> Result<(), Error> {
        if !self.fajr_angle.is_finite() || !(0.0..=90.0).contains(&self.fajr_angle) {
            return Err(Error::InvalidMethodParams {
                field: "fajr_angle",
                reason: "must be finite and between 0 and 90 degrees",
            });
        }
        if !self.isha_angle.is_finite() || !(0.0..=90.0).contains(&self.isha_angle) {
            return Err(Error::InvalidMethodParams {
                field: "isha_angle",
                reason: "must be finite and between 0 and 90 degrees",
            });
        }
        if self.isha_interval_minutes < 0 {
            return Err(Error::InvalidMethodParams {
                field: "isha_interval_minutes",
                reason: "must be non-negative",
            });
        }
        if self.maghrib_interval_minutes < 0 {
            return Err(Error::InvalidMethodParams {
                field: "maghrib_interval_minutes",
                reason: "must be non-negative",
            });
        }
        CString::new(self.name.as_str()).map_err(|_| Error::StringContainsNul)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A validated proleptic Gregorian calendar date.
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// Creates a date, returning an error if its components are invalid.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, Error> {
        if day == 0 || day > days_in_month(year, month).unwrap_or(0) {
            return Err(Error::InvalidDate { year, month, day });
        }
        Ok(Self { year, month, day })
    }

    /// Returns the year.
    pub fn year(self) -> i32 {
        self.year
    }

    /// Returns the month in the range 1 through 12.
    pub fn month(self) -> u8 {
        self.month
    }

    /// Returns the day of the month.
    pub fn day(self) -> u8 {
        self.day
    }

    /// Returns the signed number of days since 1970-01-01.
    pub fn days_since_unix_epoch(self) -> i64 {
        let month = i64::from(self.month);
        let mut year = i64::from(self.year);
        year -= i64::from(month <= 2);
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let day_of_year =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// Creates a date from a signed number of days since 1970-01-01.
    pub fn from_days_since_unix_epoch(days: i64) -> Result<Self, Error> {
        let result = (|| {
            let shifted = days.checked_add(719_468)?;
            let adjusted = if shifted >= 0 {
                shifted
            } else {
                shifted.checked_sub(146_096)?
            };
            let era = adjusted / 146_097;
            let day_of_era = shifted.checked_sub(era.checked_mul(146_097)?)?;
            let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
                - day_of_era / 146_096)
                / 365;
            let year = year_of_era.checked_add(era.checked_mul(400)?)?;
            let day_of_year =
                day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
            let month_prime = (5 * day_of_year + 2) / 153;
            let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
            let month = if month_prime < 10 {
                month_prime + 3
            } else {
                month_prime - 9
            };
            let year = year.checked_add(i64::from(month <= 2))?;

            Some((
                i32::try_from(year).ok()?,
                u8::try_from(month).ok()?,
                u8::try_from(day).ok()?,
            ))
        })()
        .ok_or(Error::DateOutOfRange(days))?;

        Self::new(result.0, result.1, result.2).map_err(|_| Error::DateOutOfRange(days))
    }
}

fn days_in_month(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if is_gregorian_leap_year(year) => Some(29),
        2 => Some(28),
        _ => None,
    }
}

fn is_gregorian_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Validated geographic coordinates in decimal degrees.
pub struct Coordinates {
    latitude: f64,
    longitude: f64,
}

impl Coordinates {
    /// Creates coordinates from latitude and longitude in decimal degrees.
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, Error> {
        if !latitude.is_finite()
            || !longitude.is_finite()
            || !(-90.0..=90.0).contains(&latitude)
            || !(-180.0..=180.0).contains(&longitude)
        {
            return Err(Error::InvalidCoordinates {
                latitude,
                longitude,
            });
        }
        Ok(Self {
            latitude,
            longitude,
        })
    }

    /// Returns the latitude in decimal degrees.
    pub fn latitude(self) -> f64 {
        self.latitude
    }

    /// Returns the longitude in decimal degrees.
    pub fn longitude(self) -> f64 {
        self.longitude
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// An explicit offset from UTC in hours.
pub struct UtcOffset {
    hours: f64,
}

impl UtcOffset {
    /// Creates an offset from a possibly fractional number of hours.
    pub fn from_hours(hours: f64) -> Result<Self, Error> {
        if !hours.is_finite() || !(-24.0..=24.0).contains(&hours) {
            return Err(Error::InvalidUtcOffset { hours });
        }
        Ok(Self { hours })
    }

    /// Returns the offset in hours.
    pub fn hours(self) -> f64 {
        self.hours
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// A calculated local prayer time represented as decimal hours.
pub struct PrayerTime(f64);

impl PrayerTime {
    /// Creates a prayer time from finite decimal hours.
    pub fn try_from_decimal_hours(decimal_hours: f64) -> Result<Self, Error> {
        Self::from_result(decimal_hours, "prayer time")
    }

    fn from_result(decimal_hours: f64, name: &'static str) -> Result<Self, Error> {
        if !decimal_hours.is_finite() {
            return Err(Error::NonFiniteResult(name));
        }
        Ok(Self(decimal_hours))
    }

    /// Returns the unrounded decimal-hour value.
    pub fn decimal_hours(self) -> f64 {
        self.0
    }

    /// Formats `HH:MM` using libprayertime's minute-ceiling convention.
    ///
    /// This may differ from [`Self::minute`], whose components are derived by
    /// rounding the complete time to the nearest second.
    pub fn format_hm(self) -> String {
        let mut output = [0 as c_char; 16];
        unsafe { ffi::format_time_hm(self.0, output.as_mut_ptr(), output.len()) };
        unsafe { CStr::from_ptr(output.as_ptr()) }
            .to_str()
            .expect("format_time_hm returned non-ASCII")
            .to_owned()
    }

    /// Formats `HH:MM:SS`, rounding to the nearest second.
    pub fn format_hms(self) -> String {
        let mut output = [0 as c_char; 16];
        unsafe { ffi::format_time_hms(self.0, output.as_mut_ptr(), output.len()) };
        unsafe { CStr::from_ptr(output.as_ptr()) }
            .to_str()
            .expect("format_time_hms returned non-ASCII")
            .to_owned()
    }

    /// Reduces onto the 24-hour clock face before decomposing.
    ///
    /// The cast below truncates toward zero, so without this a negative
    /// decimal hour yields a negative minute and second. The C header applies
    /// the same reduction in `normalize_clock_hours`, and the two must agree
    /// or `format_hms` and the component accessors disagree for the same
    /// value. `calculate` can return hours outside 0 to 24 at high latitude,
    /// so this is reachable rather than defensive.
    fn clock_hours(self) -> f64 {
        let t = self.0 % 24.0;
        if t < 0.0 { t + 24.0 } else { t }
    }

    fn hms_components(self) -> (i32, i32, i32) {
        let normalized = self.clock_hours();
        let mut hours = normalized as i32;
        let fraction = normalized - f64::from(hours);
        let total_seconds = (fraction * 3600.0 + 0.5) as i32;
        let mut minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        if minutes >= 60 {
            hours += minutes / 60;
            minutes %= 60;
        }
        hours %= 24;
        (hours, minutes, seconds)
    }

    /// Returns the hour component after rounding to the nearest second.
    pub fn hour(self) -> i32 {
        self.hms_components().0
    }

    /// Returns the minute component after rounding to the nearest second.
    ///
    /// Unlike [`Self::format_hm`], this does not apply minute ceiling.
    pub fn minute(self) -> i32 {
        self.hms_components().1
    }

    /// Returns the second component after rounding to the nearest second.
    pub fn second(self) -> i32 {
        self.hms_components().2
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The seven prayer-related times returned by a calculation.
pub struct PrayerTimes {
    /// Fajr time.
    pub fajr: PrayerTime,
    /// Sunrise time.
    pub sunrise: PrayerTime,
    /// Dhuha time.
    pub dhuha: PrayerTime,
    /// Dhuhr time.
    pub dhuhr: PrayerTime,
    /// Asr time.
    pub asr: PrayerTime,
    /// Maghrib time.
    pub maghrib: PrayerTime,
    /// Isha time.
    pub isha: PrayerTime,
}

/// Calculates all seven prayer-related times for a date and location.
///
/// `utc_offset` must be the explicit offset applicable to the requested date.
/// No time-zone database or daylight-saving rule is applied.
pub fn calculate(
    date: Date,
    coordinates: Coordinates,
    utc_offset: UtcOffset,
    params: &MethodParams,
) -> Result<PrayerTimes, Error> {
    params.validate()?;
    let name = CString::new(params.name.as_str()).map_err(|_| Error::StringContainsNul)?;
    let raw_params = ffi::MethodParams {
        name: name.as_ptr(),
        fajr_angle: params.fajr_angle,
        isha_angle: params.isha_angle,
        isha_interval: params.isha_interval_minutes,
        maghrib_interval: params.maghrib_interval_minutes,
        asr_shadow: ffi::AsrSchool::from(params.asr_school) as i32,
        midnight_mode: params.midnight_mode.into(),
        ihtiyat: params.ihtiyat_minutes,
    };
    let raw = unsafe {
        ffi::calculate_prayer_times(
            date.year(),
            date.month().into(),
            date.day().into(),
            coordinates.latitude(),
            coordinates.longitude(),
            utc_offset.hours(),
            &raw_params,
        )
    };

    Ok(PrayerTimes {
        fajr: PrayerTime::from_result(raw.fajr, "fajr")?,
        sunrise: PrayerTime::from_result(raw.sunrise, "sunrise")?,
        dhuha: PrayerTime::from_result(raw.dhuha, "dhuha")?,
        dhuhr: PrayerTime::from_result(raw.dhuhr, "dhuhr")?,
        asr: PrayerTime::from_result(raw.asr, "asr")?,
        maghrib: PrayerTime::from_result(raw.maghrib, "maghrib")?,
        isha: PrayerTime::from_result(raw.isha, "isha")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(windows, target_pointer_width = "32"))]
    fn i64_to_c_long(value: i64) -> std::ffi::c_long {
        std::ffi::c_long::try_from(value).unwrap()
    }

    #[cfg(all(not(windows), target_pointer_width = "64"))]
    fn i64_to_c_long(value: i64) -> std::ffi::c_long {
        value
    }

    #[test]
    fn prayer_time_formats_with_c_semantics() {
        let time = PrayerTime::try_from_decimal_hours(5.5002778).unwrap();
        assert_eq!(time.decimal_hours(), 5.5002778);
        assert_eq!(time.format_hm(), "05:31");
        assert_eq!(time.format_hms(), "05:30:01");
        assert_eq!((time.hour(), time.minute(), time.second()), (5, 30, 1));

        let wrapped = PrayerTime::try_from_decimal_hours(25.5).unwrap();
        assert_eq!(wrapped.format_hms(), "01:30:00");
    }

    #[test]
    fn prayer_time_components_carry_and_wrap() {
        let carry = PrayerTime::try_from_decimal_hours(5.999961111).unwrap();
        assert_eq!(carry.format_hms(), "06:00:00");
        assert_eq!((carry.hour(), carry.minute(), carry.second()), (6, 0, 0));

        let midnight = PrayerTime::try_from_decimal_hours(24.0).unwrap();
        assert_eq!(midnight.format_hms(), "00:00:00");
        assert_eq!(
            (midnight.hour(), midnight.minute(), midnight.second()),
            (0, 0, 0)
        );

        let wrapped = PrayerTime::try_from_decimal_hours(25.5).unwrap();
        assert_eq!(wrapped.format_hms(), "01:30:00");
        assert_eq!(
            (wrapped.hour(), wrapped.minute(), wrapped.second()),
            (1, 30, 0)
        );
    }

    #[test]
    fn prayer_time_rejects_non_finite_values() {
        assert!(PrayerTime::try_from_decimal_hours(f64::NAN).is_err());
        assert!(PrayerTime::try_from_decimal_hours(f64::INFINITY).is_err());
    }

    fn assert_calculation_matches_c(
        date: Date,
        coordinates: Coordinates,
        utc_offset: UtcOffset,
        params: &MethodParams,
    ) {
        let name = CString::new(params.name.as_str()).unwrap();
        let raw_params = ffi::MethodParams {
            name: name.as_ptr(),
            fajr_angle: params.fajr_angle,
            isha_angle: params.isha_angle,
            isha_interval: params.isha_interval_minutes,
            maghrib_interval: params.maghrib_interval_minutes,
            asr_shadow: match params.asr_school {
                AsrSchool::Standard => 1,
                AsrSchool::Hanafi => 2,
            },
            midnight_mode: params.midnight_mode.into(),
            ihtiyat: params.ihtiyat_minutes,
        };
        let raw = unsafe {
            ffi::calculate_prayer_times(
                date.year(),
                date.month().into(),
                date.day().into(),
                coordinates.latitude(),
                coordinates.longitude(),
                utc_offset.hours(),
                &raw_params,
            )
        };
        let safe = calculate(date, coordinates, utc_offset, params).unwrap();

        assert_eq!(safe.fajr.decimal_hours().to_bits(), raw.fajr.to_bits());
        assert_eq!(
            safe.sunrise.decimal_hours().to_bits(),
            raw.sunrise.to_bits()
        );
        assert_eq!(safe.dhuha.decimal_hours().to_bits(), raw.dhuha.to_bits());
        assert_eq!(safe.dhuhr.decimal_hours().to_bits(), raw.dhuhr.to_bits());
        assert_eq!(safe.asr.decimal_hours().to_bits(), raw.asr.to_bits());
        assert_eq!(
            safe.maghrib.decimal_hours().to_bits(),
            raw.maghrib.to_bits()
        );
        assert_eq!(safe.isha.decimal_hours().to_bits(), raw.isha.to_bits());

        assert_eq!(safe.dhuha.decimal_hours().to_bits(), raw.dhuha.to_bits());
    }

    #[test]
    fn calculation_matches_c_for_presets_and_custom_params() {
        assert_calculation_matches_c(
            Date::new(2026, 7, 30).unwrap(),
            Coordinates::new(-6.2088, 106.8456).unwrap(),
            UtcOffset::from_hours(7.0).unwrap(),
            &MethodParams::for_method(CalculationMethod::Kemenag).unwrap(),
        );
        assert_calculation_matches_c(
            Date::new(2026, 1, 15).unwrap(),
            Coordinates::new(51.5074, -0.1278).unwrap(),
            UtcOffset::from_hours(0.0).unwrap(),
            &MethodParams::for_method(CalculationMethod::Mwl).unwrap(),
        );
        assert_calculation_matches_c(
            Date::new(2026, 7, 30).unwrap(),
            Coordinates::new(21.3891, 39.8579).unwrap(),
            UtcOffset::from_hours(3.0).unwrap(),
            &MethodParams::for_method(CalculationMethod::Makkah).unwrap(),
        );

        let mut custom = MethodParams::new("Custom test");
        custom.fajr_angle = 19.5;
        custom.isha_angle = 17.5;
        custom.isha_interval_minutes = 0;
        custom.maghrib_interval_minutes = 3;
        custom.asr_school = AsrSchool::Hanafi;
        custom.midnight_mode = MidnightMode::Standard;
        custom.ihtiyat_minutes = 1;
        assert_calculation_matches_c(
            Date::new(2026, 7, 30).unwrap(),
            Coordinates::new(-6.2088, 106.8456).unwrap(),
            UtcOffset::from_hours(7.0).unwrap(),
            &custom,
        );
    }

    #[test]
    fn constants_match_c_header_values() {
        let cases = [
            (constants::DEGREES_TO_RADIANS, unsafe {
                ffi::abi_constant_deg_to_rad()
            }),
            (constants::RADIANS_TO_DEGREES, unsafe {
                ffi::abi_constant_rad_to_deg()
            }),
            (constants::JULIAN_EPOCH, unsafe {
                ffi::abi_constant_julian_epoch()
            }),
            (constants::SUN_MEAN_ANOMALY_OFFSET, unsafe {
                ffi::abi_constant_sun_mean_anomaly_offset()
            }),
            (constants::SUN_MEAN_ANOMALY_RATE, unsafe {
                ffi::abi_constant_sun_mean_anomaly_rate()
            }),
            (constants::SUN_MEAN_LONGITUDE_OFFSET, unsafe {
                ffi::abi_constant_sun_mean_longitude_offset()
            }),
            (constants::SUN_MEAN_LONGITUDE_RATE, unsafe {
                ffi::abi_constant_sun_mean_longitude_rate()
            }),
            (constants::SUN_ECCENTRICITY_AMPLITUDE_1, unsafe {
                ffi::abi_constant_sun_eccentricity_amplitude1()
            }),
            (constants::SUN_ECCENTRICITY_AMPLITUDE_2, unsafe {
                ffi::abi_constant_sun_eccentricity_amplitude2()
            }),
            (constants::OBLIQUITY_COEFFICIENT, unsafe {
                ffi::abi_constant_obliquity_coeff()
            }),
            (constants::OBLIQUITY_RATE, unsafe {
                ffi::abi_constant_obliquity_rate()
            }),
            (constants::REFRACTION_CORRECTION, unsafe {
                ffi::abi_constant_refraction_correction()
            }),
            (constants::DHUHA_ALTITUDE, unsafe {
                ffi::abi_constant_dhuha_altitude()
            }),
        ];

        for (rust_value, c_value) in cases {
            assert_eq!(rust_value.to_bits(), c_value.to_bits());
        }
    }

    #[test]
    fn validates_domain_values() {
        assert!(Date::new(2024, 2, 29).is_ok());
        assert_eq!(
            Date::new(2023, 2, 29),
            Err(Error::InvalidDate {
                year: 2023,
                month: 2,
                day: 29,
            })
        );

        assert!(Coordinates::new(-90.0, 180.0).is_ok());
        assert_eq!(
            Coordinates::new(-90.1, 0.0),
            Err(Error::InvalidCoordinates {
                latitude: -90.1,
                longitude: 0.0,
            })
        );
        assert!(matches!(
            Coordinates::new(0.0, f64::NAN),
            Err(Error::InvalidCoordinates {
                latitude: 0.0,
                longitude,
            }) if longitude.is_nan()
        ));

        assert!(UtcOffset::from_hours(24.0).is_ok());
        assert_eq!(
            UtcOffset::from_hours(24.1),
            Err(Error::InvalidUtcOffset { hours: 24.1 })
        );
    }

    #[test]
    fn civil_dates_round_trip() {
        for date in [
            Date::new(1970, 1, 1).unwrap(),
            Date::new(2000, 2, 29).unwrap(),
            Date::new(1969, 12, 31).unwrap(),
            Date::new(2026, 7, 30).unwrap(),
        ] {
            let days = date.days_since_unix_epoch();
            let c_days = unsafe {
                ffi::abi_days_from_civil(date.year(), date.month().into(), date.day().into())
            };
            assert_eq!(i128::from(days), i128::from(c_days));

            let c_days = i64_to_c_long(days);
            let (mut year, mut month, mut day) = (0, 0, 0);
            unsafe {
                ffi::abi_civil_from_days(c_days, &mut year, &mut month, &mut day);
            }
            assert_eq!(
                (year, month, day),
                (date.year(), i32::from(date.month()), i32::from(date.day()))
            );
            assert_eq!(Date::from_days_since_unix_epoch(days).unwrap(), date);
        }
    }

    #[test]
    fn every_builtin_method_loads_owned_params() {
        let methods = [
            CalculationMethod::Mwl,
            CalculationMethod::Makkah,
            CalculationMethod::Isna,
            CalculationMethod::Egypt,
            CalculationMethod::Karachi,
            CalculationMethod::Turkey,
            CalculationMethod::Singapore,
            CalculationMethod::Jakim,
            CalculationMethod::Kemenag,
            CalculationMethod::France,
            CalculationMethod::Russia,
            CalculationMethod::Dubai,
            CalculationMethod::Qatar,
            CalculationMethod::Kuwait,
            CalculationMethod::Jordan,
            CalculationMethod::Gulf,
            CalculationMethod::Tunisia,
            CalculationMethod::Algeria,
            CalculationMethod::Morocco,
            CalculationMethod::Portugal,
            CalculationMethod::Moonsighting,
        ];

        for method in methods {
            let mut params = MethodParams::for_method(method).unwrap();
            assert!(!params.name.is_empty());
            params.name.push_str(" owned");
            params.validate().unwrap();
        }
    }

    #[test]
    fn method_keys_round_trip() {
        let cases = [
            ("mwl", CalculationMethod::Mwl),
            ("makkah", CalculationMethod::Makkah),
            ("isna", CalculationMethod::Isna),
            ("egypt", CalculationMethod::Egypt),
            ("karachi", CalculationMethod::Karachi),
            ("turkey", CalculationMethod::Turkey),
            ("singapore", CalculationMethod::Singapore),
            ("jakim", CalculationMethod::Jakim),
            ("kemenag", CalculationMethod::Kemenag),
            ("france", CalculationMethod::France),
            ("russia", CalculationMethod::Russia),
            ("dubai", CalculationMethod::Dubai),
            ("qatar", CalculationMethod::Qatar),
            ("kuwait", CalculationMethod::Kuwait),
            ("jordan", CalculationMethod::Jordan),
            ("gulf", CalculationMethod::Gulf),
            ("tunisia", CalculationMethod::Tunisia),
            ("algeria", CalculationMethod::Algeria),
            ("morocco", CalculationMethod::Morocco),
            ("portugal", CalculationMethod::Portugal),
            ("moonsighting", CalculationMethod::Moonsighting),
            ("custom", CalculationMethod::Custom),
        ];
        for (key, method) in cases {
            assert_eq!(key.parse::<CalculationMethod>().unwrap(), method);
            assert_eq!(method.as_str(), key);
        }
        assert!(matches!(
            "unknown".parse::<CalculationMethod>(),
            Err(Error::UnknownCalculationMethod(_))
        ));
    }

    #[test]
    fn method_parsing_folds_case_for_every_key() {
        // C compares case-insensitively, so every key must parse in any
        // casing. "custom" is the one that regressed: it is also C's
        // not-found sentinel, so it takes a separate path in `from_str`.
        for method in ["mwl", "kemenag", "moonsighting", "custom"] {
            assert_eq!(
                method.to_ascii_uppercase().parse::<CalculationMethod>(),
                method.parse::<CalculationMethod>(),
                "casing changed the result for {method}"
            );
        }
        assert_eq!(
            "CUSTOM".parse::<CalculationMethod>().unwrap(),
            CalculationMethod::Custom
        );
        assert_eq!(
            "Custom".parse::<CalculationMethod>().unwrap(),
            CalculationMethod::Custom
        );
    }

    #[test]
    fn method_params_are_owned_and_validated() {
        let mut params = MethodParams::for_method(CalculationMethod::Kemenag).unwrap();
        assert_eq!(params.name, "KEMENAG, Indonesia");
        assert_eq!(params.fajr_angle, 20.0);
        assert_eq!(params.isha_angle, 18.0);
        assert_eq!(params.isha_interval_minutes, 0);
        assert_eq!(params.maghrib_interval_minutes, 0);
        assert_eq!(params.asr_school, AsrSchool::Standard);
        assert_eq!(params.midnight_mode, MidnightMode::Standard);
        assert_eq!(params.ihtiyat_minutes, 2);
        params.name.push_str(" (customized)");
        assert_eq!(params.name, "KEMENAG, Indonesia (customized)");
        assert!(params.validate().is_ok());

        for (field, mut invalid) in [
            ("fajr_angle", MethodParams::new("NaN Fajr")),
            ("isha_angle", MethodParams::new("NaN Isha")),
        ] {
            if field == "fajr_angle" {
                invalid.fajr_angle = f64::NAN;
            } else {
                invalid.isha_angle = f64::NAN;
            }
            assert!(matches!(
                invalid.validate(),
                Err(Error::InvalidMethodParams {
                    field: actual,
                    ..
                }) if actual == field
            ));
        }

        let mut invalid = MethodParams::new("High angle");
        invalid.fajr_angle = 90.1;
        assert!(matches!(
            invalid.validate(),
            Err(Error::InvalidMethodParams {
                field: "fajr_angle",
                ..
            })
        ));
        invalid.fajr_angle = 0.0;
        invalid.isha_angle = 90.1;
        assert!(matches!(
            invalid.validate(),
            Err(Error::InvalidMethodParams {
                field: "isha_angle",
                ..
            })
        ));

        invalid.isha_angle = 0.0;
        invalid.isha_interval_minutes = -1;
        assert!(matches!(
            invalid.validate(),
            Err(Error::InvalidMethodParams {
                field: "isha_interval_minutes",
                ..
            })
        ));
        invalid.isha_interval_minutes = 0;
        invalid.maghrib_interval_minutes = -1;
        assert!(matches!(
            invalid.validate(),
            Err(Error::InvalidMethodParams {
                field: "maghrib_interval_minutes",
                ..
            })
        ));

        let invalid = MethodParams::new("Interior\0NUL");
        assert!(matches!(invalid.validate(), Err(Error::StringContainsNul)));
    }

    #[test]
    fn components_and_formatting_agree_outside_the_day() {
        // calculate() can return hours outside 0 to 24 at high latitude, so
        // these values are reachable and not merely defensive. Before the
        // clock_hours reduction, -0.104 decomposed to (0, -6, -13).
        let cases = [
            (-0.104_f64, (23, 53, 46), "23:53:46"),
            (-24.5, (23, 30, 0), "23:30:00"),
            (25.075, (1, 4, 30), "01:04:30"),
            (24.0, (0, 0, 0), "00:00:00"),
            (12.5, (12, 30, 0), "12:30:00"),
        ];
        for (hours, want_components, want_hms) in cases {
            let t = PrayerTime::try_from_decimal_hours(hours).expect("finite");
            assert_eq!(
                (t.hour(), t.minute(), t.second()),
                want_components,
                "components for {hours}"
            );
            assert_eq!(t.format_hms(), want_hms, "format_hms for {hours}");
            // The two paths must never disagree, which is the property the
            // reduction exists to hold.
            assert_eq!(
                format!("{:02}:{:02}:{:02}", t.hour(), t.minute(), t.second()),
                t.format_hms(),
                "component and formatter disagree for {hours}"
            );
            assert!(
                (0..24).contains(&t.hour())
                    && (0..60).contains(&t.minute())
                    && (0..60).contains(&t.second()),
                "component out of range for {hours}"
            );
        }
    }
}
