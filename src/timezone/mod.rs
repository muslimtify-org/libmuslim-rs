//! Host-backed IANA time-zone resolution from libmuslim's `timezone.h`.
//!
//! This module uses the operating system's time-zone database.
//! [`crate::timezone::offset_at`] reads the zone's TZif file directly on POSIX
//! and calls `SystemTimeToTzSpecificLocalTimeEx` on Windows. It mutates no
//! process-global state, takes no lock, and is safe to call concurrently from
//! any number of threads.
//!
//! Earlier releases resolved the offset by setting the process `TZ`
//! environment variable, which raced with any other thread calling `getenv`,
//! `localtime` or `tzset`. That was fixed upstream in `timezone.h`
//! (<https://github.com/muslimtify-org/libmuslim/issues/41>) and this module
//! no longer serializes calls.
//!
//! Only IANA zone names are accepted. `timezone.h` also resolves POSIX TZ
//! strings and absolute TZif paths on POSIX, but neither form exists on
//! Windows, so [`crate::timezone::offset_at`] requires a name the host zone
//! database contains and reports
//! [`crate::timezone::TimezoneError::UnknownZone`] otherwise. That check also
//! keeps a typo from resolving: `Asia/Jakata` fails rather than answering.
//!
//! (These links are fully qualified on purpose. `src/lib.rs` puts an outer
//! doc comment on `pub mod timezone;`, which makes rustdoc resolve intra-doc
//! links in this `//!` block against the crate root rather than this module,
//! so bare names here fail the `-D warnings` doc build.)

mod ffi;

use std::ffi::{CString, c_char};

use crate::prayertimes::UtcOffset;

/// An error produced while calling `timezone.h`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TimezoneError {
    /// The supplied zone name contains an interior NUL byte.
    ZoneContainsNul,
    /// The Unix timestamp cannot be represented by the platform C `time_t`.
    TimestampOutOfRange(i64),
    /// The name is not an IANA zone the host can resolve.
    UnknownZone(String),
    /// The host system time zone could not be detected.
    SystemTimezoneUnavailable,
    /// The native function returned a string without a NUL terminator.
    MalformedNativeOutput,
    /// The native function returned a time-zone name that is not valid UTF-8.
    InvalidUtf8,
    /// The native function returned a non-finite or out-of-range UTC offset.
    InvalidOffset(f64),
}

impl std::fmt::Display for TimezoneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZoneContainsNul => {
                write!(formatter, "time-zone name contains an interior NUL byte")
            }
            Self::TimestampOutOfRange(timestamp) => write!(
                formatter,
                "Unix timestamp {timestamp} is outside the platform time_t range"
            ),
            Self::UnknownZone(zone) => {
                write!(formatter, "unknown IANA time zone: {zone}")
            }
            Self::SystemTimezoneUnavailable => {
                write!(formatter, "host system time zone is unavailable")
            }
            Self::MalformedNativeOutput => {
                write!(formatter, "native time-zone output is malformed")
            }
            Self::InvalidUtf8 => write!(formatter, "native time-zone output is not valid UTF-8"),
            Self::InvalidOffset(hours) => {
                write!(
                    formatter,
                    "native time-zone offset is invalid: {hours} hours"
                )
            }
        }
    }
}

impl std::error::Error for TimezoneError {}

/// Resolves the UTC offset for an IANA zone at a Unix timestamp.
///
/// Daylight-saving and historical changes are applied by the host operating
/// system.
///
/// Only IANA zone names are accepted. A name the host cannot resolve is
/// rejected with [`TimezoneError::UnknownZone`], so a typo cannot masquerade
/// as UTC. Bare POSIX TZ strings such as `XYZ8` and absolute paths to TZif
/// files are not IANA names and are rejected on every platform, even where
/// the underlying C library would resolve them.
///
/// This takes no lock and mutates no process-global state, so it is safe to
/// call concurrently.
pub fn offset_at(zone: &str, unix_timestamp: i64) -> Result<UtcOffset, TimezoneError> {
    let name = CString::new(zone).map_err(|_| TimezoneError::ZoneContainsNul)?;
    let mut hours = 0.0;
    let status =
        unsafe { ffi::muslim_timezone_offset_at(name.as_ptr(), unix_timestamp, &mut hours) };
    match status {
        0 => UtcOffset::from_hours(hours).map_err(|_| TimezoneError::InvalidOffset(hours)),
        -2 => Err(TimezoneError::UnknownZone(zone.to_owned())),
        _ => Err(TimezoneError::TimestampOutOfRange(unix_timestamp)),
    }
}

/// Returns the host system's IANA time-zone name.
///
/// Like [`crate::timezone::offset_at`], this touches no process-global state:
/// the native function reads `/etc/localtime` on POSIX and calls
/// `GetDynamicTimeZoneInformation` on Windows, writing only into the buffer it
/// is given.
pub fn system_timezone() -> Result<String, TimezoneError> {
    let mut output = [0 as c_char; 1024];
    let status = unsafe { ffi::get_system_timezone(output.as_mut_ptr(), output.len()) };
    if status != 0 {
        return Err(TimezoneError::SystemTimezoneUnavailable);
    }

    let end = output
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(TimezoneError::MalformedNativeOutput)?;
    let bytes: Vec<u8> = output[..end].iter().map(|byte| *byte as u8).collect();
    String::from_utf8(bytes).map_err(|_| TimezoneError::InvalidUtf8)
}
