#include <stdio.h>

#include <led.h>
#include <timer.h>
#include <button.h>
#include <led.h>

static int num_leds;
static int color = 0;
static int toggle = 0;
static volatile uint32_t ticks = 0;
static volatile uint32_t button_ticks = 0;
static uint32_t button_debounce_ticks = 3;

static void show_color(int c) {
    for (int i = 0; i < num_leds; i++) {
      if (i == c && toggle) {
        led_on(i);
      } else {
        led_off(i);
      }
    }
}

static void rotate_color(void) {
  color++;
  if (color >= num_leds)
    color = 0;

  show_color(color);
}

// Callback for button presses.
//   btn_num: The index of the button associated with the callback
//   val: 0 if pressed, 1 if depressed
static void button_callback(__attribute__ ((unused)) int btn_num,
                            int val,
                            __attribute__ ((unused)) int arg2,
                            __attribute__ ((unused)) void *ud) {
  if (val == 0) {
    if (ticks >= button_ticks + button_debounce_ticks) {
      rotate_color();
    }

    button_ticks = ticks;
  }
}

int main(void) {
  num_leds = led_count();
  printf("LEDS: %d\n", num_leds);

  button_subscribe(button_callback, NULL);

  // Enable interrupts on each button.
  int count = button_count();
  for (int i = 0; i < count; i++) {
    button_enable_interrupt(i);
  }

  while(1) {
    if (ticks % 16 == 0) {
      toggle = !toggle;
      show_color(color);
    }

    delay_ms(100);
    ticks++;
  }
}
