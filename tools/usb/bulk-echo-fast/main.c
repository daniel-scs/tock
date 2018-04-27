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
#include <sys/time.h>
#include "libusb.h"

typedef int bool;
static const bool false = 0;
static const bool true = 1;

static struct pollfd fds[10];
static const int timeout_never = -1;
static size_t stdin_fdi;

static const size_t input_bufsz = 100;
static unsigned char input_buf[input_bufsz];
static size_t input_buflen = 0;
static size_t input_buf_avail(void);
static bool input_buf_locked = false;
static size_t read_input(void);

static const uint16_t TARGET_VENDOR_ID = 0x6667;
static const uint16_t TARGET_PRODUCT_ID = 0xabcd;

unsigned char endpoint_bulk_in = 1 | 1 << 7;
unsigned char endpoint_bulk_out = 2 | 0 << 7;

static libusb_device_handle *zorp;

void configure_device(void);

static struct timeval timeval_zero = { 0, 0 };

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

    if (libusb_open(dev, &zorp))
        error(1, 0, "libusb_open");
}

void LIBUSB_CALL write_done(struct libusb_transfer *transfer) {
    switch (transfer->status) {
        case LIBUSB_TRANSFER_COMPLETED:
            if (transfer->actual_length != transfer->length) {
                error(1, 0, "short write");
            }
            input_buflen = 0;
            input_buf_locked = false;
            fprintf(stderr, "Wrote %d bytes to device\n", transfer->actual_length);
            break;
        default:
            error(1, 0, "bad transfer status: %d", transfer->status);
    }

    libusb_free_transfer(transfer);
}

void handle_events(void) {

    if (input_buflen > 0) {
        // Write input buf to device

        // Don't fiddle with input buffer while libusb is trying to send it
        input_buf_locked = true;

        int iso_packets = 0;
        struct libusb_transfer* transfer = libusb_alloc_transfer(iso_packets);
        libusb_fill_bulk_transfer(transfer, zorp, endpoint_bulk_out,
                                  input_buf, input_buflen, write_done, NULL, 0);

        if (libusb_submit_transfer(transfer))
            error(1, 0, "submit");
    }

    nfds_t nfds = 0;

    // Add stdin fd
    bool poll_stdin = !input_buf_locked && input_buf_avail() > 0;
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
    fprintf(stderr, "\t(Added %lu USB pollfds)\n", nfds - 1);

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
        // libusb must be ready

        int r = libusb_handle_events_timeout(NULL, &timeval_zero);
        if (r != 0) {
            error(1, 0, "libusb_handle_events: %s", libusb_error_name(r));
        }
    }
}

/*
 * An input buffer
 */

static size_t input_buf_avail(void) {
    return input_bufsz - input_buflen;
}

static size_t read_input(void) {
    ssize_t r = read(0, input_buf + input_buflen, input_bufsz - input_buflen);
    if (r < 0) {
        error(1, r, "read");
    }
    else {
        fprintf(stderr, "Read %ld bytes\n", r);
        input_buflen += r;
    }
    return r;
}
