#include <stdio.h>
#include <stdlib.h>
#include <timer.h>
#include <crc.h>

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

  if (crc_compute() != 0) {
    printf("CRC compute-request failed\n");
    exit(1);
  }

  if (crc_compute() != EBUSY) {
    printf("unexpected result of overlapping CRC compute-request\n");
    exit(1);
  }

  printf("Waiting for CRC results ...\n");
  while(1) {
    yield();
  }
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
