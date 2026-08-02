//! Safe, idiomatic Rust bindings for libmuslim.
//!
//! Prayer-time calculations are available through [`prayertimes`], with
//! optional host-backed IANA time-zone resolution through [`timezone`].
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

/// Host-backed IANA time-zone resolution backed by `timezone.h`.
pub mod timezone;
