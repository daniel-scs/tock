#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <led.h>
#include <spi.h>
#include <timer.h>
#include <simple_ble.h>

/*
        - App states
          - (0) WAIT (~ 10 uA)
            - on BLE connect goto (1)
          - (1) SLEEP3 (~100 uA)
            - on BLE disconnecto goto (0)
            - on receive command goto (2)
          - (2) running, sending SPI commands (many mA)
*/

enum state_t {
  Waiting,
  Listening,
  Blinking,
  Spinning,
  Failed
};

static enum state_t state;

static void await_ble_connection(void);
static void ble_connected(void);

static void await_ble_message(void);
static void ble_message_received(void);

static void await_wheel_sensor(void);
static void wheel_moving(void);

static void send_spi_messages(void);
static void led_sos(void);

int main(void) {
  await_ble_connection();
  await_wheel_sensor();
  state = Waiting;

  while (1) {
    switch (state) {
      case Waiting:
        break;

      case Listening:
        break;

      case Blinking:
        break;

      case Spinning:
        break;

      case Failed:
        // Signal failure
        led_sos();
        break;
    }

    // Wait for something interesting to happen
    yield();
  }
}

static void await_ble_connection(void)
{
  /*
  // Intervals for advertising and connections.
  // These are some basic settings for BLE devices. However, since we are
  // only interesting in scanning, these are not particularly relevant.
  simple_ble_config_t ble_config = {
    .platform_id       = 0x00, // used as 4th octet in device BLE address
    .device_id         = DEVICE_ID_DEFAULT,
    .adv_name          = "Tock",
    .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
    .min_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS),
    .max_conn_interval = MSEC_TO_UNITS(1250, UNIT_1_25_MS)
  };

  // Setup BLE.
  simple_ble_init(&ble_config);

  // Scan for advertisements.
  simple_ble_scan_start();
  */

  // Register for interrupt callback: ble_connected
}

static void ble_connected(void) {
  await_ble_message();
}

static void await_ble_message(void)
{
  state = Listening;

  // Register for callback: ble_message_received
}

static void ble_message_received(void)
{
  // We've been asked to blink the lights
  send_spi_messages();
}

static void await_wheel_sensor(void)
{
  // Register for interrupt callback
}

static void wheel_moving(void) {
  // Begin animation
}

static void send_spi_messages(void) {
  state = Blinking;

  spi_set_chip_select(0);
  spi_set_rate(400000);
  spi_set_polarity(false);
  spi_set_phase(false);

#define BUF_SIZE 100
  static char buf[BUF_SIZE];

  // XX: Write lighting message into buf 

  int r = spi_write_sync(buf, BUF_SIZE);
  if (r != 0) {
    state = Failed;
  }

  state = Waiting;
}

// Signal failure
#define L 0
static void led_sos(void) {
    while (1) {
      led_on(L);  delay_ms(25);
      led_off(L); delay_ms(25);
      led_on(L);  delay_ms(25);
      led_off(L); delay_ms(25);
      led_on(L);  delay_ms(25);
      led_off(L); delay_ms(25);

      led_on(L);  delay_ms(100);
      led_off(L); delay_ms(100);
      led_on(L);  delay_ms(100);
      led_off(L); delay_ms(100);
      led_on(L);  delay_ms(100);
      led_off(L); delay_ms(100);
    }
}
