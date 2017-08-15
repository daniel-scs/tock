//! Provides userspace applications with the ability to encrypt and decrypt
//! messages.
//!
//! Userspace Interface
//! -------------------
//!
//! The system calls allow, subscribe and command are used to initiate the
//! driver. The methods `set_key_done()` and `crypt_done()` are invoked by chip
//! to send back the result of the operation and then passed the user
//! application via the callback from the subscribe call.
//!
//! ### `allow` System Call
//!
//! The `allow` system call is used to provide three different buffers and the
//! following allow_num's are supported:
//!
//! * 0: A buffer with the key to be used for encryption and decryption.
//! * 1: A buffer with data that will be encrypted and/or decrypted
//! * 4: A buffer to configure to initial counter when counter mode of block
//!   cipher is used.
//!
//! The possible return codes from the 'allow' system call indicate the
//! following:
//!
//! * `SUCCESS`: The buffer has successfully been filled.
//! * `ENOSUPPORT`: Invalid allow_num.
//! * `ENOMEM`: No sufficient memory available.
//! * `EINVAL`: Invalid address of the buffer or other error.
//!
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will receive the result of
//! configuring the key, encryption or decryption. The possible return from the
//! `subscribe` system call indicates the following:
//!
//! * `SUCCESS`: the callback been successfully been configured.
//! * `ENOSUPPORT`: Invalid allow_num.
//! * `ENOMEM`: No sufficient memory available.
//! * `EINVAL`: Invalid address of the buffer or other error.
//!
//!
//! ### `command` System Call
//!
//! The `command` system call supports two arguments `cmd` and `sub_cmd`. `cmd`
//! is used to specify the specific operation, currently the following cmd's are
//! supported:
//!
//! * `2`: encryption
//! * `3`: decryption
//!
//! `sub_cmd` is used to specify the specific algorithm to be used and currently
//!  the following sub_cmd's are supported:
//!
//! * `0`: aes128 counter-mode
//!
//! The possible return from the 'command' system call indicates the following:
//!
//! * `SUCCESS`:    The operation has been successful.
//! * `EBUSY`:      The driver is busy.
//! * `ESIZE`:      Invalid key size currently is must be 16, 24 or 32 bytes.
//! * `ENOSUPPORT`: Invalid `cmd` or `sub_cmd`.
//! * `EFAIL`:      The key is configured or other error.

// Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
// Author: Fredrik Nilsson <frednils@student.chalmers.se>
// Date: March 31, 2017

use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption::{AES128Ctr, AES128_BLOCK_SIZE, Client};
use kernel::process::Error;

pub struct Request {
    encrypting: bool,
    start_index: usize,
    stop_index: usize,
}

pub struct App {
    callback: Option<Callback>,
    key: Option<AppSlice<Shared, u8>>,
    iv: Option<AppSlice<Shared, u8>>,
    data: Option<AppSlice<Shared, u8>>,

    // If Some, the process has requested an encryption operation
    request: Option<Request>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            key: None,
            iv: None,
            data: None,
            request: None,
        }
    }
}

pub struct Crypto<'a, E: AES128Ctr + 'a> {
    encryptor: &'a E,
    apps: Container<App>,
    serving_app: Cell<Option<AppId>>,
}

impl<'a, E: AES128Ctr + 'a> Crypto<'a, E> {
    pub fn new(encryptor: &'a E, container: Container<App>) -> Crypto<'a, E> {
        Crypto {
            encryptor: encryptor,
            apps: container,
            serving_app: Cell::new(None),
        }
    }

    // Register a process's request for an encryption computation
    fn request(&self, appid: AppId, encrypting: bool) -> ReturnCode {
        let result = self.apps.enter(appid, |app, _| {
                if app.request.is_some() {
                    // Each app may make only one request at a time
                    ReturnCode::EBUSY
                } else {
                    if app.callback.is_some() &&
                       app.key.is_some() &&
                       app.iv.is_some() &&
                       app.data.is_some()
                    {
                        app.request = Some(Request {
                                             encrypting: encrypting,
                                             start_index: 0,
                                             stop_index: app.data.unwrap().len(),
                                           });
                        ReturnCode::SUCCESS
                    } else {
                        ReturnCode::EINVAL
                    }
                }
            })
            .unwrap_or_else(|err| match err {
                Error::OutOfMemory => ReturnCode::ENOMEM,
                Error::AddressOutOfBounds => ReturnCode::EINVAL,
                Error::NoSuchApp => ReturnCode::EINVAL,
            });

        if result == ReturnCode::SUCCESS {
            self.serve_waiting_apps();
        }
        result
    }

    fn serve_waiting_apps(&self) {
        if self.serving_app.get().is_some() {
            // A computation is in progress
            return;
        }

        // Find a waiting app and start its requested computation
        for app in self.apps.iter() {
            app.enter(|app, _| {
                if let Some(request) = app.request {
                    // A failing result that needs to be sent back to the app
                    let mut result = Some(ReturnCode::EINVAL);

                    if let Some(key) = app.key.take() {
                    if let Some(iv) = app.iv.take() {
                    if let Some(data) = app.data.take() {
                        let (r, data_opt) = self.encryptor.crypt(self,
                                                                 request.encrypting,
                                                                 key.as_ref(),
                                                                 iv.as_ref(),
                                                                 data.as_mut(),
                                                                 request.start_index,
                                                                 request.stop_index);
                        app.key = Some(key);
                        app.iv = Some(iv);
                        app.data = Some(data);

                        if r == ReturnCode::SUCCESS && data_opt.is_none() {
                            // The encryptor is now performing the encryption
                            self.serving_app.set(Some(app.appid()));

                            // Don't send a callback until it is complete
                            result = None;
                        } else {
                            // Remove the failed request
                            app.request = None;

                            // Replace the taken data buffer
                            if let Some(data) = data_opt {
                                // XXX app.data = Some(data);
                            }

                            // Arrange for an immediate callback
                            result = Some(r);
                        }
                    }}}

                    if let Some(result) = result {
                        // Try to return the failing result
                        if let Some(mut callback) = app.callback {
                            callback.schedule(From::from(result), 0, 0);
                        }
                    }
                }
            });

            if self.serving_app.get().is_some() {
                break;
            }
        }
    }
}

impl<'a, E: AES128Ctr + 'a> Client for Crypto<'a, E> {
    fn crypt_done(&self, data: &'static mut [u8]) {
        if let Some(appid) = self.serving_app.get() {
            self.apps
                .enter(appid, |app, _| {
                    // Restore taken buffer
                    // XXX app.data = Some(data);

                    if let Some(mut callback) = app.callback {
                        callback.schedule(From::from(ReturnCode::SUCCESS), 0, 0);
                    }
                    app.request = None;
                })
                .unwrap_or_else(|err| match err {
                    Error::OutOfMemory => {}
                    Error::AddressOutOfBounds => {}
                    Error::NoSuchApp => {}
                });

            self.serving_app.set(None);
            self.serve_waiting_apps();
        } else {
            // Ignore orphaned computation
        }
    }
}

impl<'a, E: AES128Ctr> Driver for Crypto<'a, E> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        // Register one of three buffers: key, data, iv
        match allow_num {
            0 => {
                if slice.len() != AES128_BLOCK_SIZE {
                    ReturnCode::EINVAL
                } else {
                    self.apps
                        .enter(appid, |app, _| {
                            app.key = Some(slice);
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| match err {
                            Error::OutOfMemory => ReturnCode::ENOMEM,
                            Error::AddressOutOfBounds => ReturnCode::EINVAL,
                            Error::NoSuchApp => ReturnCode::EINVAL,
                        })
                }
            }
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.data = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            4 => {
                if slice.len() != AES128_BLOCK_SIZE {
                    ReturnCode::EINVAL
                } else {
                    self.apps
                        .enter(appid, |app, _| {
                            app.iv = Some(slice);
                            ReturnCode::SUCCESS
                        })
                        .unwrap_or_else(|err| match err {
                            Error::OutOfMemory => ReturnCode::ENOMEM,
                            Error::AddressOutOfBounds => ReturnCode::EINVAL,
                            Error::NoSuchApp => ReturnCode::EINVAL,
                        })
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // Register callback
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
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, cmd: usize, sub_cmd: usize, appid: AppId) -> ReturnCode {
        match cmd {
            // Request encryption (sub_cmd indicates algorithm)
            2 => {
                match sub_cmd {
                    0 => self.request(appid, true),
                    _ => ReturnCode::ENOSUPPORT,
                }
            }

            // Request decryption (sub_cmd indicates algorithm)
            3 => {
                match sub_cmd {
                    0 => self.request(appid, false),
                    _ => ReturnCode::ENOSUPPORT,
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
