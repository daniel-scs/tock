//! CRC driver

// TODO: virtualize/automate init()?

use kernel::{AppId, AppSlice, Container, Callback, Driver, ReturnCode, Shared};
use kernel::hil;
use kernel::process::Error;

pub struct App {
    callback: Option<Callback>,
    buffer: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            buffer: None,
        }
    }
}

pub struct Crc<'a, C: hil::crc::CRC + 'a> {
    crc_unit: &'a C,
    apps: Container<App>,
}

impl<'a, C: hil::crc::CRC> Crc<'a, C> {
    pub fn new(crc_unit: &'a C, apps: Container<App>) -> Crc<'a, C> {
        Crc { crc_unit: crc_unit,
              apps: apps,
            }
    }
}

impl<'a, C: hil::crc::CRC> Driver for Crc<'a, C>  {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            // Provide user buffer to compute CRC over
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.buffer = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // Set callback for CRC result
            0 => {
                self.apps
                    .enter(callback.app_id(), |app, _| {
                        app.callback = Some(callback);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _data: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // The driver is present
            0 => ReturnCode::SUCCESS,

            // Get version
            1 => ReturnCode::SuccessWithValue {
                                value: self.crc_unit.get_version() as usize
                             },

            // Initialize the unit
            2 => self.crc_unit.init(),

            // Request a CRC computation
            3 => {
                self.apps
                    .enter(appid, |app, _| {
                        if app.callback.is_some() {
                            if let Some(ref buf) = app.buffer {
                                self.crc_unit.compute(buf.as_ref());
                                ReturnCode::SUCCESS
                            }
                            else { ReturnCode::EINVAL }
                        }
                        else { ReturnCode::EINVAL }
                    })
                    .unwrap_or_else(|err| {
                        match err {
                            Error::OutOfMemory => ReturnCode::ENOMEM,
                            Error::AddressOutOfBounds => ReturnCode::EINVAL,
                            Error::NoSuchApp => ReturnCode::EINVAL,
                        }
                    })
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, C: hil::crc::CRC> hil::crc::Client for Crc<'a, C> {
    fn receive_result(&self, result: u32) {
        for app in self.apps.iter() {
            app.enter(|app, _| {
                // XXX: for now just alert every app
                if let Some(mut callback) = app.callback {
                    callback.schedule(result as usize, 0, 0);
                }
            });
        }
    }
}
