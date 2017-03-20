#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_CRC 12

int crc_exists(void);
int crc_version(void);

#ifdef __cplusplus
}
#endif

