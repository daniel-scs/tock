#include <stdbool.h>

#include <stdio.h>
#include <led.h>
#include <spi.h>
#include <timer.h>

#define BUF_SIZE 200
char rbuf[BUF_SIZE];
char wbuf[BUF_SIZE];
bool toggle = true;
bool got_callback = false;

static void sos_loop(void) {
    while (1) {
      led_toggle(0);
      delay_ms(25);
      led_toggle(0);
      delay_ms(25);
      led_toggle(0);
      delay_ms(25);

      led_toggle(0);
      delay_ms(100);
      led_toggle(0);
      delay_ms(100);
      led_toggle(0);
      delay_ms(100);
    }
}

static int spi_read_write_x(const char* write, char* read, size_t len, subscribe_cb cb, bool* cond) {
  int r;

  // Signal we are about to request a transaction
  led_set(0);

  r = spi_read_write(write, read, len, cb, cond);

  if (r != 0) {
    // Signal failure
    sos_loop();
  }

  return r;
}

static void write_cb(__attribute__ ((unused)) int arg0,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) int arg3,
                     __attribute__ ((unused)) void* userdata) {

  // Diagnostics
  led_clear(0);
  if (got_callback == false) {
    printf("*** Got SPI callback!\n");
    got_callback = true;
  }

  // Start another transaction
  delay_ms(25);
  if (toggle) {
    spi_read_write_x(rbuf, wbuf, BUF_SIZE, write_cb, NULL);
  } else {
    spi_read_write_x(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
  }
  toggle = !toggle;
}

// This function can operate in one of two modes. Either
// a periodic timer triggers an SPI operation, or SPI
// operations are performed back-to-back (callback issues
// the next one.) The periodic one writes 6 byte messages,
// the back-to-back writes a 10 byte message, followed by
// 6 byte ones.
//
// In both cases, the calls alternate on which of two
// buffers is used as the write buffer. The first call
// uses the buffer initialized to 0..199. The
// 2n calls use the buffer initialized to 0.
//
// If you use back-to-back operations, the calls
// both read and write. Periodic operations only
// write. Therefore, if you set SPI to loopback
// and use back-to-back // loopback, then the read buffer
// on the first call will read in the data written.  As a
// result, you can check if reads work properly: all writes
// will be 0..n rather than all 0s.

int main(void) {
  led_clear(0);

  int i;
  for (i = 0; i < BUF_SIZE; i++) {
    wbuf[i] = i;
  }
  spi_set_chip_select(0);
  // spi_set_rate(400000);
  // spi_set_rate(40000);
  spi_set_polarity(false);
  spi_set_phase(false);

  spi_read_write_x(wbuf, rbuf, BUF_SIZE, write_cb, NULL);

  printf("*** Made SPI request\n");
}
