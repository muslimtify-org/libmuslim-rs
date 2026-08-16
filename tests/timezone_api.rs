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
fn rejects_zone_names_the_host_cannot_resolve() {
    // The whole point: a typo must not come back as a plausible UTC offset.
    assert_eq!(
        offset_at("Asia/Jakata", 0),
        Err(TimezoneError::UnknownZone("Asia/Jakata".into()))
    );
    // A bare POSIX TZ string has no tzdb entry. glibc would happily parse
    // "XYZ8" as "8 hours west of UTC"; the zone database is the authority
    // here, so it is rejected. (Note that "EST5EDT" looks like a POSIX string
    // but is a real legacy tzdb file, so it resolves -- membership in the
    // database is the rule, not the shape of the name.)
    assert_eq!(
        offset_at("XYZ8", 0),
        Err(TimezoneError::UnknownZone("XYZ8".into()))
    );
    // A name must never be usable to reach outside the zone database.
    assert_eq!(
        offset_at("../../etc/passwd", 0),
        Err(TimezoneError::UnknownZone("../../etc/passwd".into()))
    );
    assert_eq!(
        offset_at("", 0),
        Err(TimezoneError::UnknownZone(String::new()))
    );
}

#[test]
fn concurrent_lookups_of_different_zones_stay_correct() {
    // `offset_at` dropped its mutex once muslimtify-org/libmuslim#41 was fixed
    // upstream, so this asserts the answers survive without it.
    //
    // It is not a reproduction of #41 and would not have failed before the
    // fix: that race needed an unsynchronized second party reading `TZ`, and
    // the old mutex serialized every caller reachable from this crate. The
    // reproduction lives upstream, next to the code that was racing. What this
    // catches is a shim that reintroduces process-global state now that
    // nothing on the Rust side is serializing it.
    const ITERATIONS: usize = 50_000;
    let cases = [("America/New_York", -5.0), ("Asia/Jakarta", 7.0)];

    let wrong: usize = std::thread::scope(|scope| {
        let handles: Vec<_> = cases
            .iter()
            .map(|(zone, expected)| {
                scope.spawn(move || {
                    (0..ITERATIONS)
                        .filter(|_| offset_at(zone, 0).unwrap().hours() != *expected)
                        .count()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .sum()
    });

    assert_eq!(wrong, 0);
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
