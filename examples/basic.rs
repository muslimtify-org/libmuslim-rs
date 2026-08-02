use std::error::Error;

use libmuslim::prayertimes::{
    AsrSchool, CalculationMethod, Coordinates, Date, MethodParams, MidnightMode, PrayerTimes,
    calculate,
};
use libmuslim::timezone::offset_at;

fn print_times(label: &str, times: &PrayerTimes) {
    println!("{label}");
    println!("  Fajr:    {}", times.fajr.format_hm());
    println!("  Sunrise: {}", times.sunrise.format_hm());
    println!("  Dhuha:   {}", times.dhuha.format_hm());
    println!("  Dhuhr:   {}", times.dhuhr.format_hm());
    println!("  Asr:     {}", times.asr.format_hm());
    println!("  Maghrib: {}", times.maghrib.format_hm());
    println!("  Isha:    {}", times.isha.format_hm());
}

fn main() -> Result<(), Box<dyn Error>> {
    let date = Date::new(2026, 7, 30)?;
    let coordinates = Coordinates::new(-6.2, 106.8)?;
    println!("Calculating prayer times for {date:?} at {coordinates:?}");

    // Resolve the offset through the host timezone database. Supplying an
    // explicit UtcOffset remains available when the offset is already known.
    let utc_offset = offset_at("Asia/Jakarta", 1_785_369_600)?;

    let kemenag = MethodParams::for_method(CalculationMethod::Kemenag)?;
    let times = calculate(date, coordinates, utc_offset, &kemenag)?;
    print_times("Kemenag preset", &times);

    let mut adjusted = MethodParams::for_method(CalculationMethod::Kemenag)?;
    adjusted.name = "Adjusted Kemenag".into();
    adjusted.ihtiyat_minutes = 3;
    let times = calculate(date, coordinates, utc_offset, &adjusted)?;
    print_times("Adjusted Kemenag preset", &times);

    let mut custom = MethodParams::new("Custom example");
    custom.fajr_angle = 18.0;
    custom.isha_angle = 17.0;
    custom.isha_interval_minutes = 0;
    custom.maghrib_interval_minutes = 0;
    custom.asr_school = AsrSchool::Standard;
    custom.midnight_mode = MidnightMode::Standard;
    custom.ihtiyat_minutes = 1;
    let times = calculate(date, coordinates, utc_offset, &custom)?;
    print_times("Fully custom method", &times);

    Ok(())
}
