#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_LPS25HB 11

int lps25hb_set_callback (subscribe_cb callback, void* callback_args);
int lps25hb_get_pressure ();

int lps25hb_get_pressure_sync ();

#ifdef __cplusplus
}
#endif
