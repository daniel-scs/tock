//! A test of the USBC
//!
//! Creates a `SimpleClient` and sets it as the client
//! of the USB hardware interface.

use kernel::hil::usb::Client;
use capsules;
use sam4l;

pub fn test() {

    let buf = unsafe {
        static mut BUF: [u8; 8] = [0xee; 8];
        &mut BUF
    };
    let p = buf.as_mut_ptr();
    debug!("*** BUF @ {:?} / {:8x}", p, p as u32);

    unsafe {
        let client = static_init!(
            capsules::usb_simple::SimpleClient<'static, sam4l::usbc::Usbc<'static>>,
            capsules::usb_simple::SimpleClient::new(&sam4l::usbc::USBC), 224/8);

        sam4l::usbc::USBC.set_client(client);

        client.enable();
        client.attach();
    }
}
