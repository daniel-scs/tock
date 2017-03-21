#include <stdio.h>

#include <led.h>
#include <timer.h>
#include <crc.h>

void blinken(uint32_t rate);

int main(void) {
  uint32_t v;

  if (!crc_exists()) {
    printf("CRC driver does not exist\n");
    goto fail;
  }

  v = crc_version();
  if (v != 0x00000202) {
    printf("CRC version unexpected: %lu\n", v);
    goto fail;
  }

  if (crc_init() != 0) {
    printf("CRC init failed\n");
    goto fail;
  }

  printf("CRC SUCCESS\n");
  blinken(1000);

fail:
  printf("CRC FAIL\n");
  blinken(100);
}

__attribute__((noreturn)) void blinken(uint32_t rate)
{
  while(1) {
    led_on(0);
    delay_ms(rate);
    led_off(0);
    delay_ms(rate);
  }
}
