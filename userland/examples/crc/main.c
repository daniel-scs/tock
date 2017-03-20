#include <stdio.h>

#include <led.h>
#include <timer.h>
#include <crc.h>

int main(void) {
  uint32_t pause = 100;

  if (crc_exists()) {
    if (crc_init() == 0) {
      if (crc_version() == 0x00000202) {
        pause = 1000;
      }
    }
    else {
      pause = 500;
    }
  }

  int num_leds = led_count();
  for (int count = 0; ; count++) {
    for (int i = 0; i < num_leds; i++) {
      if (count & (1 << i)) {
        led_on(i);
      } else {
        led_off(i);
      }
    }
    delay_ms(pause);
  }
}
