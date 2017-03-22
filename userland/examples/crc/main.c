#include <stdio.h>
#include <stdlib.h>
#include <timer.h>
#include <crc.h>

static char buf[] = "ABCDEFG";
static size_t buf_len = sizeof(buf) - 1;

void finished(int val, int v1, int v2, void *data);

int main(void) {
  uint32_t v;

  if (!crc_exists()) {
    printf("CRC driver does not exist\n");
    exit(1);
  }

  v = crc_version();
  if (v != 0x00000202) {
    printf("CRC version unexpected: %lu\n", v);
    exit(1);
  }

  if (crc_init() != 0) {
    printf("CRC init failed\n");
    exit(1);
  }

  if (crc_subscribe(finished, 0) !=0) {
    printf("CRC subscribe failed\n");
    exit(1);
  }

  if (crc_set_buffer(buf, buf_len) != 0) {
    printf("CRC set-buffer failed\n");
    exit(1);
  }

  if (crc_compute(CRC_CCIT16) != 0) {
    printf("CRC compute-request failed\n");
    exit(1);
  }

  printf("Waiting for CRC results ...\n");
  while(1) {
    yield();
  }
}

void finished(int v0, __attribute__((unused)) int v1,
                      __attribute__((unused)) int v2,
                      __attribute__((unused)) void *data) {

  uint32_t val = v0;
  printf("CRC finished: %8lx\n", (uint32_t) val);

  if (val != 0xffff1541) {
    printf("incorrect CRC result!\n");
    exit(1);
  }

  delay_ms(1000);
  if (crc_compute(CRC_CCIT16) != 0) {
    printf("additional CRC compute-request failed\n");
  }
}
