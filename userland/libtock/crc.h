#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_CRC 12

int       crc_exists(void);
uint32_t  crc_version(void);
int       crc_init(void);
int       crc_subscribe(subscribe_cb, void *);
int       crc_set_buffer(const void*, size_t);

enum crc_polynomial {
  CRC_CCIT8023,   // Polynomial 0x04C11DB7
  CRC_CASTAGNOLI, // Polynomial 0x1EDC6F41
  CRC_CCIT16      // Polynomial 0x1021
};

int crc_compute(enum crc_polynomial);

#ifdef __cplusplus
}
#endif

