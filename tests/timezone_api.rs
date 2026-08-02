use libmuslim::prayertimes::{CalculationMethod, Coordinates, Date, MethodParams, calculate};
use libmuslim::timezone::{TimezoneError, offset_at, system_timezone};

#[test]
fn resolves_fixed_and_daylight_saving_offsets() {
    let jakarta = offset_at("Asia/Jakarta", 1_774_180_800).unwrap();
    assert_eq!(jakarta.hours(), 7.0);

    let london_winter = offset_at("Europe/London", 1_768_478_400).unwrap();
    let london_summer = offset_at("Europe/London", 1_784_116_800).unwrap();
    assert_eq!(london_winter.hours(), 0.0);
    assert_eq!(london_summer.hours(), 1.0);

    let new_york_winter = offset_at("America/New_York", 1_768_478_400).unwrap();
    let new_york_summer = offset_at("America/New_York", 1_784_116_800).unwrap();
    assert_eq!(new_york_winter.hours(), -5.0);
    assert_eq!(new_york_summer.hours(), -4.0);
}

#[test]
fn accepts_zero_offset_and_rejects_interior_nul() {
    assert_eq!(offset_at("UTC", 0).unwrap().hours(), 0.0);
    assert_eq!(
        offset_at("Europe/London\0invalid", 0),
        Err(TimezoneError::ZoneContainsNul)
    );
}

#[test]
fn detects_a_nonempty_system_timezone_or_reports_native_failure() {
    match system_timezone() {
        Ok(zone) => assert!(!zone.is_empty()),
        Err(error) => assert_eq!(error, TimezoneError::SystemTimezoneUnavailable),
    }
}

#[test]
fn resolved_offset_feeds_prayer_time_calculation() {
    let date = Date::new(2026, 7, 15).unwrap();
    let coordinates = Coordinates::new(-6.2, 106.8).unwrap();
    let offset = offset_at("Asia/Jakarta", 1_784_073_600).unwrap();
    let params = MethodParams::for_method(CalculationMethod::Kemenag).unwrap();

    let times = calculate(date, coordinates, offset, &params).unwrap();
    assert!(times.fajr.decimal_hours().is_finite());
    assert!(times.isha.decimal_hours().is_finite());
}
