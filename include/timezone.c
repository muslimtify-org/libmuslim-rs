#define MUSLIM_TIMEZONE_IMPLEMENTATION
#include "timezone.h"

#include <stdint.h>

int muslim_timezone_offset_at(const char *tz_name, int64_t unix_timestamp,
                              double *offset) {
  time_t when = (time_t)unix_timestamp;
  if ((int64_t)when != unix_timestamp)
    return -1;

  *offset = parse_timezone_offset(tz_name, when);
  return 0;
}
