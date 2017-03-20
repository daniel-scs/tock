#include "crc.h"

int crc_exists(void) {
  return command(DRIVER_NUM_CRC, 0, 0) >= 0;
}

int crc_version(void) {
  return command(DRIVER_NUM_CRC, 1, 0);
}

