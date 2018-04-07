#include <stdint.h>
#include <stdio.h>

#include <internal/nonvolatile_storage.h>

static int test(uint8_t *readbuf, uint8_t *writebuf, size_t size, size_t offset, size_t len);

static bool done = false;

static void read_done(int length,
                      __attribute__ ((unused)) int arg1,
                      __attribute__ ((unused)) int arg2,
                      __attribute__ ((unused)) void* ud) {
  printf("Finished read! %i\n", length);
  done = true;
}

static void write_done(int length,
                       __attribute__ ((unused)) int arg1,
                       __attribute__ ((unused)) int arg2,
                       __attribute__ ((unused)) void* ud) {
  printf("Finished write! %i\n", length);
  done = true;
}

int main (void) {
  int r;
  uint8_t readbuf[512];
  uint8_t writebuf[512];

  printf("[Nonvolatile Storage] Test App\n");

  int num_bytes = nonvolatile_storage_internal_get_number_bytes();
  printf("Have %i bytes of nonvolatile storage\n", num_bytes);

  if ((r = test(readbuf, writebuf, 256, 0,  14)) != 0) return r;
  if ((r = test(readbuf, writebuf, 256, 20, 14)) != 0) return r;
  if ((r = test(readbuf, writebuf, 512, 0, 512)) != 0) return r;

  printf("\tAll tests succeeded.\n");

  return 0;
}

static int test(uint8_t *readbuf, uint8_t *writebuf, size_t size, size_t offset, size_t len) {
  int ret;

  printf("\tTest with size %d\n", size);

  ret = nonvolatile_storage_internal_read_buffer(readbuf, size);
  if (ret != 0) {
    printf("ERROR setting read buffer\n");
    return ret;
  }

  ret = nonvolatile_storage_internal_write_buffer(writebuf, size);
  if (ret != 0) {
    printf("ERROR setting write buffer\n");
    return ret;
  }

  // Setup callbacks
  ret = nonvolatile_storage_internal_read_done_subscribe(read_done, NULL);
  if (ret != 0) {
    printf("ERROR setting read done callback\n");
    return ret;
  }

  ret = nonvolatile_storage_internal_write_done_subscribe(write_done, NULL);
  if (ret != 0) {
    printf("ERROR setting write done callback\n");
    return ret;
  }

  for (size_t i = offset; i < offset + len; i++) {
    writebuf[i] = i;
  }

  done = false;
  ret = nonvolatile_storage_internal_write(offset, len);
  if (ret != 0) {
    printf("ERROR calling write\n");
    return ret;
  }
  yield_for(&done);

  done = false;
  ret  = nonvolatile_storage_internal_read(offset, len);
  if (ret != 0) {
    printf("ERROR calling read\n");
    return ret;
  }
  yield_for(&done);

  for (size_t i = offset; i < offset + len; i++) {
    if (readbuf[i] != writebuf[i]) {
      printf("Inconsistency between data written and read at index %u\n", i);
      return -1;
    }
  }

  return 0;
}
