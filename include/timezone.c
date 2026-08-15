#define MUSLIM_TIMEZONE_IMPLEMENTATION
#include "timezone.h"

#include <stdint.h>
#include <string.h>

/* parse_timezone_offset() returns 0.0 both for a genuine UTC zone and for a
 * name the host cannot resolve, so a typo like "Asia/Jakata" is indis-
 * tinguishable from real UTC at the call site. That ambiguity is fixed here by
 * checking the name against the host's zone database up front, so callers get
 * an explicit failure instead of a plausible-looking wrong offset.
 *
 * Only IANA zone names are accepted. POSIX TZ strings ("EST5EDT", "UTC-7")
 * resolve on Unix but have no tzdb entry and cannot be expressed on Windows,
 * so they are rejected on both platforms to keep the behaviour identical. */

/* Reject anything that is not a plausible IANA zone name before it is used to
 * build a filesystem path: no absolute paths, no "..", no empty components,
 * and only the characters IANA actually uses. */
static int muslim_zone_name_is_safe(const char *tz_name) {
  if (!tz_name || tz_name[0] == '\0' || tz_name[0] == '/')
    return 0;
  if (strstr(tz_name, "..") != NULL || strstr(tz_name, "//") != NULL)
    return 0;
  if (strlen(tz_name) > 255)
    return 0;

  for (const char *p = tz_name; *p != '\0'; p++) {
    unsigned char c = (unsigned char)*p;
    int ok = (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ||
             (c >= '0' && c <= '9') || c == '_' || c == '-' || c == '+' ||
             c == '/';
    if (!ok)
      return 0;
  }
  if (tz_name[strlen(tz_name) - 1] == '/')
    return 0;
  return 1;
}

#if defined(_WIN32)

/* Windows has no tzdb on disk. The two places parse_timezone_offset() can fail
 * to resolve a name are the bundled IANA<->Windows table and the dynamic zone
 * enumeration, so both are checked here. */
int muslim_timezone_zone_exists(const char *tz_name) {
  if (!muslim_zone_name_is_safe(tz_name))
    return 0;

  const wchar_t *win_zone = muslim_iana_to_windows_zone(tz_name);
  if (!win_zone)
    return 0;

  DYNAMIC_TIME_ZONE_INFORMATION dtzi;
  DWORD idx = 0;
  while (EnumDynamicTimeZoneInformation(idx++, &dtzi) == ERROR_SUCCESS) {
    if (wcscmp(dtzi.TimeZoneKeyName, win_zone) == 0)
      return 1;
  }
  return 0;
}

#else /* !_WIN32 */

#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>

/* A zone resolves on POSIX exactly when libc can find its file under the zone
 * database directory, which is what tzset() consults. TZDIR overrides the
 * compiled-in default in glibc and musl, so honour it too. */
int muslim_timezone_zone_exists(const char *tz_name) {
  if (!muslim_zone_name_is_safe(tz_name))
    return 0;

  const char *tzdir = getenv("TZDIR");
  if (!tzdir || tzdir[0] == '\0')
    tzdir = "/usr/share/zoneinfo";

  char path[1024];
  int written = snprintf(path, sizeof(path), "%s/%s", tzdir, tz_name);
  if (written < 0 || (size_t)written >= sizeof(path))
    return 0;

  struct stat info;
  if (stat(path, &info) != 0)
    return 0;
  return S_ISREG(info.st_mode) ? 1 : 0;
}

#endif /* _WIN32 */

/* Returns 0 on success, -1 if the timestamp is outside the platform time_t
 * range, -2 if the zone is not a resolvable IANA name. */
int muslim_timezone_offset_at(const char *tz_name, int64_t unix_timestamp,
                              double *offset) {
  time_t when = (time_t)unix_timestamp;
  if ((int64_t)when != unix_timestamp)
    return -1;

  if (!muslim_timezone_zone_exists(tz_name))
    return -2;

  *offset = parse_timezone_offset(tz_name, when);
  return 0;
}
