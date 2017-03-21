#include <stdio.h>

#include <led.h>
#include <timer.h>
#include <crc.h>

int main(void) {
  uint32_t pause = 100;
  uint32_t v;

  if (!crc_exists()) {
    printf("CRC driver does not exist\n");
    goto fail;
  }

  if (crc_init() != 0) {
    printf("CRC init failed\n");
    goto fail;
  }

  v = crc_version();
  if (v != 0x00000202) {
    printf("CRC version unexpected: %lu\n", v);
    goto fail;
  }

  pause = 1000;
  printf("CRC SUCCESS\n");
  goto blinken;

fail:
  printf("CRC FAIL\n");

blinken:
  while(1) {
    led_on(0);
    delay_ms(pause);
    led_off(0);
    delay_ms(pause);
  }
}
