#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <timer.h>
#include <crc.h>

struct test_case {
  enum crc_polynomial poly;
  uint32_t output;
  char *input;
};

#define CASE(poly, output, input) char input_ ## poly ## _ ## output [] = input;
#include "test_cases.h"
#undef CASE

static struct test_case test_cases[] = {
#define CASE(poly, output, input) \
  { poly, output, input_ ## poly ## _ ## output },
#include "test_cases.h"
#undef CASE
};

int n_test_cases = sizeof(test_cases) / sizeof(struct test_case);

int test_index;

void receive_result(int, int, int, void *);

bool completed;

int main(void) {
  int r;

  if (!crc_exists()) {
    printf("CRC driver does not exist\n");
    exit(1);
  }

  uint32_t v = crc_version();
  if (v != 0x00000202) {
    printf("CRC version unexpected: %lu\n", v);
    exit(1);
  }

  if (crc_subscribe(receive_result, 0) !=0) {
    printf("CRC subscribe failed\n");
    exit(1);
  }

  for (test_index = 0; test_index < n_test_cases; test_index++) {
    struct test_case *t = &test_cases[test_index];

    printf("Requesting test case %d (length %d) ...\n",
           test_index, strlen(t->input));

    if ((r = crc_set_buffer(t->input, strlen(t->input))) != 0) {
      printf("CRC set-buffer failed: %d\n", r);
      exit(1);
    }

    completed = false;
    if ((r = crc_compute(t->poly)) != 0) {
      printf("CRC compute-request failed: %d\n", r);
      exit(1);
    }

    printf("Waiting for CRC results ...\n");
    yield_for(&completed);
  }

  printf("Finished\n");
}

void receive_result(int v0, int v1,
                    __attribute__((unused)) int v2,
                    __attribute__((unused)) void *data)
{
  usize status = v0;
  uint32_t result = v1;

  struct test_case *t = &test_cases[test_index];

  printf("-> Case %d: ", test_index);
  if (status == SUCCESS) {
    printf("result=%8lx ", result);
    if (result == t->output)
      printf("(OK)");
    else
      printf("(Expected %8lx)", t->output);
  }
  else {
    printf("failed with status %d\n", status);
  }
  printf("\n");

  completed = true;
}
