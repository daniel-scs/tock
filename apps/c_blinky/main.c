#include <firestorm.h>
#include <gpio.h>

CB_TYPE timer_cb(int, int, int, void*);

void main(void) {
    gpio_enable_output(LED_0);
    timer_subscribe(timer_cb, NULL);
    timer_start_repeating(500);
}

CB_TYPE timer_cb(int arg0, int arg2, int arg3, void* userdata) {
  gpio_toggle(LED_0);
  return 0;
}

