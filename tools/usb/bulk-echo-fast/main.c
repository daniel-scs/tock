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

void configure_device(void);

static bool done = false;
void handle_events(void);

int main(void) {
    configure_device();

    while (!done) {
        handle_events();
    }

    fprintf(stderr, "Done\n");
    return 0;
}

void configure_device(void) {
    libusb_device **devs;
    int r;
    ssize_t cnt;

    r = libusb_init(NULL);
    if (r < 0)
        error(1, r, "libusb_init");

    cnt = libusb_get_device_list(NULL, &devs);
    if (cnt < 0)
        error(1, (int) cnt, "libusb_get_device_list");

    libusb_device *dev;
    int i = 0;
    while ((dev = devs[i++]) != NULL) {
        struct libusb_device_descriptor desc;
        int r = libusb_get_device_descriptor(dev, &desc);
        if (r < 0)
            error(1, r, "failed to get device descriptor");

        if (desc.idVendor == TARGET_VENDOR_ID &&
            desc.idProduct == TARGET_PRODUCT_ID)
            break;
    }
    if (dev == NULL)
        error(1, 0, "Couldn't find target device");
}

void handle_events(void) {

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
    const struct libusb_pollfd **all_usb_fds = libusb_get_pollfds(NULL);
    if (all_usb_fds == NULL) {
        error(1, 0, "libusb_get_pullfds");
    }
    for (const struct libusb_pollfd **usb_fds = all_usb_fds; *usb_fds != NULL; usb_fds++) {
        const struct libusb_pollfd *pollfd = *usb_fds;

        fds[nfds].fd = pollfd->fd;
        fds[nfds].events = pollfd->events;
        fds[nfds].revents = 0;
        nfds++;
    }
    libusb_free_pollfds(all_usb_fds);
    fprintf(stderr, "Added %lu USB pollfds\n", nfds - 1);

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
