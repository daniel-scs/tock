#include <stdio.h>
#include <stdint.h>
#include "libusb.h"

static const uint16_t TARGET_VENDOR_ID = 0x6667;
static const uint16_t TARGET_PRODUCT_ID = 0xabcd;

static void (libusb_device **devs)
{
	libusb_device *dev;
	int i = 0, j = 0;
	uint8_t path[8]; 

	while ((dev = devs[i++]) != NULL) {
		struct libusb_device_descriptor desc;
		int r = libusb_get_device_descriptor(dev, &desc);
		if (r < 0) {
			fprintf(stderr, "failed to get device descriptor");
			return;
		}

                if (desc.idVendor == TARGET_VENDOR_ID &&
                    desc.idProduct == TARGET_PRODUCT_ID) {
		printf("%04x:%04x (bus %d, device %d)",
			desc.idVendor, desc.idProduct,
	}
}

int main(void)
{
	libusb_device **devs;
	int r;
	ssize_t cnt;
          

	r = libusb_init(NULL);
	if (r < 0)
		return r;

	cnt = libusb_get_device_list(NULL, &devs);
	if (cnt < 0)
		return (int) cnt;

	print_devs(devs);
	libusb_free_device_list(devs, 1);

	libusb_exit(NULL);
	return 0;
}
