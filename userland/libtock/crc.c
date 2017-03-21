#include "crc.h"

int crc_exists(void) {
  return command(DRIVER_NUM_CRC, 0, 0) >= 0;
}

uint32_t crc_version(void) {
  return command(DRIVER_NUM_CRC, 1, 0);
}

int crc_init(void) {
  return command(DRIVER_NUM_CRC, 2, 0);
}

int crc_subscribe(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_NUM_CRC, 0, callback, ud);
}

int crc_compute(void) {
  return command(DRIVER_NUM_CRC, 3, 0);
}
