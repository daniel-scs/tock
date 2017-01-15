//! This allows a 9DOF sensor to be used by multiple apps.
//!
//! The data from the driver is not virtualized. This just gives apps
//! exclusive access to the driver until a callback occurs.

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver};
use kernel::hil;
use kernel::returncode::ReturnCode;

pub struct App {
    callback: Option<Callback>,
    pending_command: bool,
    command: usize,
    arg1: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            pending_command: false,
            command: 0,
            arg1: 0,
        }
    }
}

pub struct VirtualNineDof<'a> {
    driver: &'a hil::ninedof::NineDof,
    apps: Container<App>,
    current_app: Cell<Option<AppId>>,
}

impl<'a> VirtualNineDof<'a> {
    pub fn new(driver: &'a hil::ninedof::NineDof, container: Container<App>) -> VirtualNineDof<'a> {
        VirtualNineDof {
            driver: driver,
            apps: container,
            current_app: Cell::new(None),
        }
    }

    fn enqueue_command(&self, command_num: usize, arg1: usize, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                // Check so see if we are doing something. If not,
                // go ahead and do this command. If so, this is queued
                // and will be run when the pending command completes.
                if self.current_app.get().is_none() {
                    self.current_app.set(Some(appid));
                    self.call_driver(command_num, arg1)
                } else {
                    if app.pending_command == true {
                        ReturnCode::ENOMEM
                    } else {
                        app.pending_command = true;
                        app.command = command_num;
                        app.arg1 = arg1;
                        ReturnCode::SUCCESS
                    }
                }
            })
            .unwrap_or(ReturnCode::FAIL)
    }

    fn call_driver(&self, command_num: usize, _: usize) -> ReturnCode {
        match command_num {
            1 => self.driver.read_accelerometer(),
            100 => self.driver.read_magnetometer(),
            _ => ReturnCode::FAIL,
        }
    }
}

impl<'a> hil::ninedof::NineDofClient for VirtualNineDof<'a> {
    fn callback(&self, arg1: usize, arg2: usize, arg3: usize) {
        // Notify the current application that the command finished.
        self.current_app.get().map(|appid| {
            self.current_app.set(None);
            let _ = self.apps.enter(appid, |app, _| {
                app.pending_command = false;
                app.callback.map(|mut cb| {
                    cb.schedule(arg1, arg2, arg3);
                });
            });
        });

        // Check if there are any pending events.
        for cntr in self.apps.iter() {
            let started_command = cntr.enter(|app, _| {
                if app.pending_command {
                    app.pending_command = false;
                    self.current_app.set(Some(app.appid()));
                    self.call_driver(app.command, app.arg1) == ReturnCode::SUCCESS
                } else {
                    false
                }
            });
            if started_command {
                break;
            }
        }
    }
}

impl<'a> Driver for VirtualNineDof<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                self.apps
                    .enter(callback.app_id(), |app, _| {
                        app.callback = Some(callback);
                        0
                    })
                    .unwrap_or(-1)
            }
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, arg1: usize, appid: AppId) -> isize {
        match command_num {
            0 => /* This driver exists. */ 0,

            // Single acceleration reading.
            1 => {
                let err = self.enqueue_command(command_num, arg1, appid);
                (err as isize) * -1
            }

            // Single magnetometer reading.
            100 => {
                let err = self.enqueue_command(command_num, arg1, appid);
                (err as isize) * -1
            }

            _ => -1,
        }
    }
}
