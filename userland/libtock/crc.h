#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_CRC 12

int       crc_exists(void);
uint32_t  crc_version(void);
int       crc_init(void);

#ifdef __cplusplus
}
#endif

