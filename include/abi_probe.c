#include <stddef.h>
#include "prayertimes.h"

_Static_assert(CALC_MWL == 0, "CALC_MWL");
_Static_assert(CALC_MAKKAH == 1, "CALC_MAKKAH");
_Static_assert(CALC_ISNA == 2, "CALC_ISNA");
_Static_assert(CALC_EGYPT == 3, "CALC_EGYPT");
_Static_assert(CALC_KARACHI == 4, "CALC_KARACHI");
_Static_assert(CALC_TURKEY == 5, "CALC_TURKEY");
_Static_assert(CALC_SINGAPORE == 6, "CALC_SINGAPORE");
_Static_assert(CALC_JAKIM == 7, "CALC_JAKIM");
_Static_assert(CALC_KEMENAG == 8, "CALC_KEMENAG");
_Static_assert(CALC_FRANCE == 9, "CALC_FRANCE");
_Static_assert(CALC_RUSSIA == 10, "CALC_RUSSIA");
_Static_assert(CALC_DUBAI == 11, "CALC_DUBAI");
_Static_assert(CALC_QATAR == 12, "CALC_QATAR");
_Static_assert(CALC_KUWAIT == 13, "CALC_KUWAIT");
_Static_assert(CALC_JORDAN == 14, "CALC_JORDAN");
_Static_assert(CALC_GULF == 15, "CALC_GULF");
_Static_assert(CALC_TUNISIA == 16, "CALC_TUNISIA");
_Static_assert(CALC_ALGERIA == 17, "CALC_ALGERIA");
_Static_assert(CALC_MOROCCO == 18, "CALC_MOROCCO");
_Static_assert(CALC_PORTUGAL == 19, "CALC_PORTUGAL");
_Static_assert(CALC_MOONSIGHTING == 20, "CALC_MOONSIGHTING");
_Static_assert(CALC_CUSTOM == 21, "CALC_CUSTOM");
_Static_assert(CALC_COUNT == 22, "CALC_COUNT");
_Static_assert(ASR_STANDARD == 1 && ASR_HANAFI == 2, "AsrSchool");
_Static_assert(HIGHLAT_NONE == 0 && HIGHLAT_MIDDLE_OF_NIGHT == 1 &&
                   HIGHLAT_ONE_SEVENTH == 2 && HIGHLAT_ANGLE_BASED == 3,
               "HighLatMethod");
_Static_assert(MIDNIGHT_STANDARD == 0, "MidnightMode");

size_t abi_sizeof_method_params(void) { return sizeof(MethodParams); }
size_t abi_alignof_method_params(void) { return _Alignof(MethodParams); }
size_t abi_offsetof_method_params_name(void) { return offsetof(MethodParams, name); }
size_t abi_offsetof_method_params_fajr_angle(void) { return offsetof(MethodParams, fajr_angle); }
size_t abi_offsetof_method_params_isha_angle(void) { return offsetof(MethodParams, isha_angle); }
size_t abi_offsetof_method_params_isha_interval(void) { return offsetof(MethodParams, isha_interval); }
size_t abi_offsetof_method_params_maghrib_interval(void) { return offsetof(MethodParams, maghrib_interval); }
size_t abi_offsetof_method_params_asr_shadow(void) { return offsetof(MethodParams, asr_shadow); }
size_t abi_offsetof_method_params_midnight_mode(void) { return offsetof(MethodParams, midnight_mode); }
size_t abi_offsetof_method_params_ihtiyat(void) { return offsetof(MethodParams, ihtiyat); }

size_t abi_sizeof_prayer_times(void) { return sizeof(struct PrayerTimes); }
size_t abi_alignof_prayer_times(void) { return _Alignof(struct PrayerTimes); }
size_t abi_offsetof_prayer_times_fajr(void) { return offsetof(struct PrayerTimes, fajr); }
size_t abi_offsetof_prayer_times_dhuhr(void) { return offsetof(struct PrayerTimes, dhuhr); }
size_t abi_offsetof_prayer_times_asr(void) { return offsetof(struct PrayerTimes, asr); }
size_t abi_offsetof_prayer_times_maghrib(void) { return offsetof(struct PrayerTimes, maghrib); }
size_t abi_offsetof_prayer_times_isha(void) { return offsetof(struct PrayerTimes, isha); }

double abi_constant_deg_to_rad(void) { return DEG_TO_RAD; }
double abi_constant_rad_to_deg(void) { return RAD_TO_DEG; }
double abi_constant_julian_epoch(void) { return JULIAN_EPOCH; }
double abi_constant_sun_mean_anomaly_offset(void) { return SUN_MEAN_ANOMALY_OFFSET; }
double abi_constant_sun_mean_anomaly_rate(void) { return SUN_MEAN_ANOMALY_RATE; }
double abi_constant_sun_mean_longitude_offset(void) { return SUN_MEAN_LONGITUDE_OFFSET; }
double abi_constant_sun_mean_longitude_rate(void) { return SUN_MEAN_LONGITUDE_RATE; }
double abi_constant_sun_eccentricity_amplitude1(void) { return SUN_ECCENTRICITY_AMPLITUDE1; }
double abi_constant_sun_eccentricity_amplitude2(void) { return SUN_ECCENTRICITY_AMPLITUDE2; }
double abi_constant_obliquity_coeff(void) { return OBLIQUITY_COEFF; }
double abi_constant_obliquity_rate(void) { return OBLIQUITY_RATE; }
double abi_constant_refraction_correction(void) { return REFRACTION_CORRECTION; }

long abi_days_from_civil(int y, int m, int d) {
  return mt_days_from_civil(y, m, d);
}

void abi_civil_from_days(long days, int *y, int *m, int *d) {
  mt_civil_from_days(days, y, m, d);
}

/* Pin the public prototypes.
 *
 * src/prayertimes/ffi.rs declares these by hand -- no bindgen -- so a changed
 * signature in prayertimes.h would otherwise compile clean on both sides and
 * only misbehave at run time. Initialising a pointer of the expected type makes
 * any mismatch a build error instead.
 *
 * These are deliberately not `static`: an unused static would trip
 * -Wunused-const-variable on every build. Nothing reads them; the type check at
 * initialisation is the whole point.
 *
 * This pins the header against this file, not against ffi.rs. When one of these
 * fails to compile, the Rust extern block needs updating too. */
struct PrayerTimes (*abi_fn_calculate)(int, int, int, double, double, double,
                                       const MethodParams *) =
    calculate_prayer_times;
const MethodParams *(*abi_fn_params_get)(CalcMethod) = method_params_get;
CalcMethod (*abi_fn_from_string)(const char *) = method_from_string;
const char *(*abi_fn_to_string)(CalcMethod) = method_to_string;
void (*abi_fn_format_hm)(double, char *, size_t) = format_time_hm;
void (*abi_fn_format_hms)(double, char *, size_t) = format_time_hms;
