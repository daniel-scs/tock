/* This utility requires that the cross-platform (Windows, OSX, Linux) library
 * [libusb](http://libusb.info/) is installed on the host machine.
 *
 * NOTE: This code uses libusb interfaces not available on Windows.
 */
#include <stdio.h>
#include <stdint.h>
#include <unistd.h>
#include <poll.h>
#include <error.h>
#include "libusb.h"

typedef int bool;
static const bool false = 0;
static const bool true = 1;

static const uint16_t TARGET_VENDOR_ID = 0x6667;
static const uint16_t TARGET_PRODUCT_ID = 0xabcd;

static struct pollfd fds[10];
static const int timeout_never = -1;
static size_t stdin_fdi;

static size_t input_buf_avail(void);
static size_t read_input(void);

static bool done = false;

int main(void) {
    while (!done) {
        nfds_t nfds = 0;

        // Add stdin fd
        bool poll_stdin = input_buf_avail() > 0;
        if (poll_stdin) {
            fds[nfds].fd = 0;
            fds[nfds].events = POLLIN;
            fds[nfds].revents = 0;
            stdin_fdi = nfds;
            nfds++;
        }

        // Add libusb fds

        if (nfds == 0) {
            // Nothing to wait for
            error(1, 0, "Deadlocked");
        }

        // Poll for ready fds
        int nfds_active = poll(fds, nfds, timeout_never);
        if (nfds_active < 0) {
            error(1, nfds_active, "poll");
        }

        // Check if stdin ready
        if (poll_stdin) {
            if (fds[stdin_fdi].revents != 0) {
                if (read_input() == 0) {
                  done = true;
                }
                nfds_active--;
            }
        }

        if (nfds_active > 0) {
            fprintf(stderr, "Other things ready\n");
        }
    }
    fprintf(stderr, "Done\n");
}

/*
 * An input buffer
 */

static const size_t input_bufsz = 100;
static size_t input_buflen = 0;

static size_t input_buf_avail(void) {
    return input_bufsz - input_buflen;
}

static size_t read_input(void) {
    static char buf[input_bufsz];

    ssize_t r = read(0, buf + input_buflen, input_bufsz - input_buflen);
    if (r < 0) {
        error(1, r, "read");
    }
    else {
        fprintf(stderr, "Read %ld bytes\n", r);
        input_buflen += r;
    }
    return r;
}
