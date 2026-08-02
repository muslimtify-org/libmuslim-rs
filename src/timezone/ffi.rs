use std::ffi::{c_char, c_double, c_int};

unsafe extern "C" {
    pub(super) fn muslim_timezone_offset_at(
        tz_name: *const c_char,
        unix_timestamp: i64,
        offset: *mut c_double,
    ) -> c_int;

    pub(super) fn get_system_timezone(buf: *mut c_char, cap: usize) -> c_int;
}
