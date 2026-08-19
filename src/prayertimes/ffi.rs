#![allow(dead_code)]

#[cfg(test)]
use std::ffi::c_long;
use std::ffi::{c_char, c_double, c_int};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CalcMethod {
    Mwl = 0,
    Makkah,
    Isna,
    Egypt,
    Karachi,
    Turkey,
    Singapore,
    Jakim,
    Kemenag,
    France,
    Russia,
    Dubai,
    Qatar,
    Kuwait,
    Jordan,
    Gulf,
    Tunisia,
    Algeria,
    Morocco,
    Portugal,
    Moonsighting,
    Custom,
    Count,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AsrSchool {
    Standard = 1,
    Hanafi = 2,
}

/// Mirror of the C `HighLatMethod` enum.
///
/// This was previously kept only to pin discriminants, on the expectation that
/// upstream would one day add a `high_lat_method` field to `MethodParams`.
/// libmuslim v0.2.0 did exactly that, so the enum is now read, surfaced through
/// the safe API as [`crate::prayertimes::HighLatMethod`], and the pinning kept.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HighLatMethod {
    None = 0,
    MiddleOfNight,
    OneSeventh,
    AngleBased,
    NearestLatitude,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MidnightMode {
    Standard = 0,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct MethodParams {
    pub(crate) name: *const c_char,
    pub(crate) fajr_angle: c_double,
    pub(crate) isha_angle: c_double,
    pub(crate) isha_interval: c_int,
    pub(crate) maghrib_interval: c_int,
    pub(crate) asr_shadow: c_int,
    pub(crate) midnight_mode: MidnightMode,
    pub(crate) ihtiyat: c_int,
    pub(crate) high_lat_method: HighLatMethod,
    pub(crate) high_lat_ref: c_double,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct PrayerTimes {
    pub(crate) fajr: c_double,
    pub(crate) dhuhr: c_double,
    pub(crate) asr: c_double,
    pub(crate) maghrib: c_double,
    pub(crate) isha: c_double,
}

unsafe extern "C" {
    pub(crate) fn format_time_hm(time_hours: c_double, out: *mut c_char, len: usize);
    pub(crate) fn format_time_hms(time_hours: c_double, out: *mut c_char, len: usize);
    pub(crate) fn method_params_get(method: CalcMethod) -> *const MethodParams;
    pub(crate) fn method_from_string(name: *const c_char) -> CalcMethod;
    pub(crate) fn method_to_string(method: CalcMethod) -> *const c_char;
    pub(crate) fn calculate_prayer_times(
        year: c_int,
        month: c_int,
        day: c_int,
        latitude: c_double,
        longitude: c_double,
        timezone: c_double,
        params: *const MethodParams,
    ) -> PrayerTimes;
}

#[cfg(test)]
unsafe extern "C" {
    pub(crate) fn abi_constant_deg_to_rad() -> c_double;
    pub(crate) fn abi_constant_rad_to_deg() -> c_double;
    pub(crate) fn abi_constant_julian_epoch() -> c_double;
    pub(crate) fn abi_constant_sun_mean_anomaly_offset() -> c_double;
    pub(crate) fn abi_constant_sun_mean_anomaly_rate() -> c_double;
    pub(crate) fn abi_constant_sun_mean_longitude_offset() -> c_double;
    pub(crate) fn abi_constant_sun_mean_longitude_rate() -> c_double;
    pub(crate) fn abi_constant_sun_eccentricity_amplitude1() -> c_double;
    pub(crate) fn abi_constant_sun_eccentricity_amplitude2() -> c_double;
    pub(crate) fn abi_constant_obliquity_coeff() -> c_double;
    pub(crate) fn abi_constant_obliquity_rate() -> c_double;
    pub(crate) fn abi_constant_refraction_correction() -> c_double;
    pub(crate) fn abi_days_from_civil(year: c_int, month: c_int, day: c_int) -> c_long;
    pub(crate) fn abi_civil_from_days(
        days: c_long,
        year: *mut c_int,
        month: *mut c_int,
        day: *mut c_int,
    );
}

#[cfg(test)]
mod tests {
    use std::mem::{align_of, offset_of, size_of};

    use super::*;

    unsafe extern "C" {
        fn abi_sizeof_method_params() -> usize;
        fn abi_alignof_method_params() -> usize;
        fn abi_offsetof_method_params_name() -> usize;
        fn abi_offsetof_method_params_fajr_angle() -> usize;
        fn abi_offsetof_method_params_isha_angle() -> usize;
        fn abi_offsetof_method_params_isha_interval() -> usize;
        fn abi_offsetof_method_params_maghrib_interval() -> usize;
        fn abi_offsetof_method_params_asr_shadow() -> usize;
        fn abi_offsetof_method_params_midnight_mode() -> usize;
        fn abi_offsetof_method_params_ihtiyat() -> usize;
        fn abi_sizeof_prayer_times() -> usize;
        fn abi_alignof_prayer_times() -> usize;
        fn abi_offsetof_prayer_times_fajr() -> usize;
        fn abi_offsetof_prayer_times_dhuhr() -> usize;
        fn abi_offsetof_prayer_times_asr() -> usize;
        fn abi_offsetof_prayer_times_maghrib() -> usize;
        fn abi_offsetof_prayer_times_isha() -> usize;
    }

    fn assert_layout(actual: usize, expected: unsafe extern "C" fn() -> usize) {
        assert_eq!(actual, unsafe { expected() });
    }

    #[test]
    fn method_params_layout_matches_c() {
        assert_layout(size_of::<MethodParams>(), abi_sizeof_method_params);
        assert_layout(align_of::<MethodParams>(), abi_alignof_method_params);
        assert_layout(
            offset_of!(MethodParams, name),
            abi_offsetof_method_params_name,
        );
        assert_layout(
            offset_of!(MethodParams, fajr_angle),
            abi_offsetof_method_params_fajr_angle,
        );
        assert_layout(
            offset_of!(MethodParams, isha_angle),
            abi_offsetof_method_params_isha_angle,
        );
        assert_layout(
            offset_of!(MethodParams, isha_interval),
            abi_offsetof_method_params_isha_interval,
        );
        assert_layout(
            offset_of!(MethodParams, maghrib_interval),
            abi_offsetof_method_params_maghrib_interval,
        );
        assert_layout(
            offset_of!(MethodParams, asr_shadow),
            abi_offsetof_method_params_asr_shadow,
        );
        assert_layout(
            offset_of!(MethodParams, midnight_mode),
            abi_offsetof_method_params_midnight_mode,
        );
        assert_layout(
            offset_of!(MethodParams, ihtiyat),
            abi_offsetof_method_params_ihtiyat,
        );
    }

    #[test]
    fn prayer_times_layout_matches_c() {
        assert_layout(size_of::<PrayerTimes>(), abi_sizeof_prayer_times);
        assert_layout(align_of::<PrayerTimes>(), abi_alignof_prayer_times);
        assert_layout(
            offset_of!(PrayerTimes, fajr),
            abi_offsetof_prayer_times_fajr,
        );
        assert_layout(
            offset_of!(PrayerTimes, dhuhr),
            abi_offsetof_prayer_times_dhuhr,
        );
        assert_layout(offset_of!(PrayerTimes, asr), abi_offsetof_prayer_times_asr);
        assert_layout(
            offset_of!(PrayerTimes, maghrib),
            abi_offsetof_prayer_times_maghrib,
        );
        assert_layout(
            offset_of!(PrayerTimes, isha),
            abi_offsetof_prayer_times_isha,
        );
    }

    #[test]
    fn enum_discriminants_match_c() {
        assert_eq!(CalcMethod::Mwl as c_int, 0);
        assert_eq!(CalcMethod::Makkah as c_int, 1);
        assert_eq!(CalcMethod::Isna as c_int, 2);
        assert_eq!(CalcMethod::Egypt as c_int, 3);
        assert_eq!(CalcMethod::Karachi as c_int, 4);
        assert_eq!(CalcMethod::Turkey as c_int, 5);
        assert_eq!(CalcMethod::Singapore as c_int, 6);
        assert_eq!(CalcMethod::Jakim as c_int, 7);
        assert_eq!(CalcMethod::Kemenag as c_int, 8);
        assert_eq!(CalcMethod::France as c_int, 9);
        assert_eq!(CalcMethod::Russia as c_int, 10);
        assert_eq!(CalcMethod::Dubai as c_int, 11);
        assert_eq!(CalcMethod::Qatar as c_int, 12);
        assert_eq!(CalcMethod::Kuwait as c_int, 13);
        assert_eq!(CalcMethod::Jordan as c_int, 14);
        assert_eq!(CalcMethod::Gulf as c_int, 15);
        assert_eq!(CalcMethod::Tunisia as c_int, 16);
        assert_eq!(CalcMethod::Algeria as c_int, 17);
        assert_eq!(CalcMethod::Morocco as c_int, 18);
        assert_eq!(CalcMethod::Portugal as c_int, 19);
        assert_eq!(CalcMethod::Moonsighting as c_int, 20);
        assert_eq!(CalcMethod::Custom as c_int, 21);
        assert_eq!(CalcMethod::Count as c_int, 22);
        assert_eq!(AsrSchool::Standard as c_int, 1);
        assert_eq!(AsrSchool::Hanafi as c_int, 2);
        assert_eq!(HighLatMethod::None as c_int, 0);
        assert_eq!(HighLatMethod::MiddleOfNight as c_int, 1);
        assert_eq!(HighLatMethod::OneSeventh as c_int, 2);
        assert_eq!(HighLatMethod::AngleBased as c_int, 3);
        assert_eq!(HighLatMethod::NearestLatitude as c_int, 4);
        assert_eq!(MidnightMode::Standard as c_int, 0);
    }
}
