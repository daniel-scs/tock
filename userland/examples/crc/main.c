#include <stdio.h>

#include <led.h>
#include <timer.h>
#include <crc.h>

void blinken(uint32_t rate);
void finished(int val, int v1, int v2, void *data);

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

  if (crc_subscribe(finished, 0) !=0) {
    printf("CRC subscribe failed\n");
    goto fail;
  }

  if (crc_compute() != 0) {
    printf("CRC compute-request failed\n");
    goto fail;
  }

  if (crc_compute() != EBUSY) {
    printf("unexpected result of overlapping CRC compute-request\n");
  }

  printf("CRC SUCCESS\n");
  blinken(1000);

fail:
  printf("CRC FAIL\n");
  blinken(100);
}

void finished(int val, __attribute__((unused)) int v1,
                       __attribute__((unused)) int v2,
                       __attribute__((unused)) void *data) {
  printf("CRC finished: %8lx\n", (uint32_t) val);

  delay_ms(1000);

  if (crc_compute() != 0) {
    printf("additional CRC compute-request failed\n");
  }
}

__attribute__((noreturn)) void
blinken(uint32_t rate)
{
  while(1) {
    led_on(0);
    delay_ms(rate);
    led_off(0);
    delay_ms(rate);
  }
}
