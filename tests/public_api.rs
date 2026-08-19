use std::str::FromStr;

use libmuslim::prayertimes::{
    AsrSchool, CalculationMethod, Coordinates, Date, MethodParams, MidnightMode, PrayerTime,
    PrayerTimes, UtcOffset, calculate, constants,
};

#[test]
fn external_consumer_can_use_the_complete_safe_surface() {
    let date = Date::new(2026, 7, 30).unwrap();
    assert_eq!(
        Date::from_days_since_unix_epoch(date.days_since_unix_epoch()).unwrap(),
        date
    );

    let coordinates = Coordinates::new(-6.2, 106.8).unwrap();
    let offset = UtcOffset::from_hours(7.0).unwrap();
    let method = CalculationMethod::from_str("kemenag").unwrap();
    assert_eq!(method.as_str(), "kemenag");

    let mut params = MethodParams::for_method(method).unwrap();
    params.name = "Custom Kemenag".into();
    params.asr_school = AsrSchool::Hanafi;
    params.midnight_mode = MidnightMode::Standard;
    params.validate().unwrap();

    let times: PrayerTimes = calculate(date, coordinates, offset, &params).unwrap();
    let values: [PrayerTime; 5] = [
        times.fajr,
        times.dhuhr,
        times.asr,
        times.maghrib,
        times.isha,
    ];
    assert!(values.iter().all(|time| time.decimal_hours().is_finite()));
    assert!(values.iter().all(|time| time.format_hm().len() == 5));

    assert_eq!(constants::REFRACTION_CORRECTION, 0.833);
}
