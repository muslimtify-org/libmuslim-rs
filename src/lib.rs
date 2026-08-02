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
