//! A test of the USBC
//!
//! Creates a `SimpleClient` and sets it as the client
//! of the USB hardware interface.

use kernel::hil::usb::Client;
use capsules;
use sam4l;

pub fn test() {
    unsafe {
        let client = static_init!(
            capsules::usb_simple::SimpleClient<'static, sam4l::usbc::Usbc<'static>>,
            capsules::usb_simple::SimpleClient::new(&sam4l::usbc::USBC), 192/8);

        sam4l::usbc::USBC.set_client(client);

        client.enable();
        client.attach();
    }
}
