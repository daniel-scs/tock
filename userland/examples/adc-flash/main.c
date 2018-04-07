#include <stdio.h>
#include <stdlib.h>
#include <timer.h>
#include <internal/nonvolatile_storage.h>
#include <adc.h>

#define BUFSIZE 512

static uint8_t buf[BUFSIZE];

static bool write_done = false;

static void write_cb(__attribute__ ((unused)) int length,
                     __attribute__ ((unused)) int arg1,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) void* ud) {
    write_done = true;
}

int main(void) {
    int ret;

    printf("Begin test\n");

    ret = nonvolatile_storage_internal_write_buffer(buf, BUFSIZE);
    if (ret != 0) {
        printf("ERROR setting write buffer: %d\n", ret);
        exit(1);
    }

    ret = nonvolatile_storage_internal_write_done_subscribe(write_cb, NULL);
    if (ret != 0) {
        printf("ERROR setting write done callback\n");
        exit(1);
    }
    
    while(1){
        write_done = false;
        ret = nonvolatile_storage_internal_write(0, BUFSIZE);
        if (ret != 0) {
            printf("ERROR calling write: %d\n", ret);
            exit(1);
        }
        yield_for(&write_done);
        printf("Write success\n");

        delay_ms(1000);
    }
}
