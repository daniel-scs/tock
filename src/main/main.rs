#![feature(core,no_std)]
#![no_main]
#![no_std]

extern crate core;
extern crate common;
extern crate support;
extern crate hil;
extern crate process;
extern crate platform;

mod spi_test;

pub mod syscall;

#[no_mangle]
pub extern fn main() {
    use core::prelude::*;
    use process::Process;
    use common::shared::Shared;

    use hil::spi_master::SPI;

    let mut platform = unsafe {
        platform::init()
    };

    // SPI test
    platform.spi_master.init(hil::spi_master::SPIParams {
        baud_rate: 9600,
        data_order: hil::spi_master::DataOrder::LSBFirst,
        clock_polarity: hil::spi_master::ClockPolarity::IdleHigh,
        clock_phase: hil::spi_master::ClockPhase::SampleLeading,
    });
    platform.spi_master.enable_tx();
    platform.spi_master.enable_rx();
    platform.spi_master.write(&[0b10101010], || {});

    let app1 = unsafe { Process::create(spi_test::spi_test::_start).unwrap() };

    let mut processes = [Shared::new(app1)];

    loop {
        // Testing SPI

        platform.spi_master.write(&[0b10101010], || {});

        unsafe {
            platform.service_pending_interrupts();

            'sched: for process_s in processes.iter_mut() {
                let process = process_s.borrow_mut();
                'process: loop {
                    match process.state {
                        process::State::Running => {
                            process.switch_to();
                        }
                        process::State::Waiting => {
                            match process.callbacks.dequeue() {
                                None => { continue 'sched },
                                Some(cb) => {
                                    process.state = process::State::Running;
                                    process.switch_to_callback(cb);
                                }
                            }
                        }
                    }
                    match process.svc_number() {
                        Some(syscall::WAIT) => {
                            process.state = process::State::Waiting;
                            process.pop_syscall_stack();
                            break 'process;
                        },
                        Some(syscall::SUBSCRIBE) => {
                            let driver_num = process.r0();
                            let subdriver_num = process.r1();
                            let callback_ptr = process.r2() as *mut ();

                            let res = platform.with_driver(driver_num, |driver| {
                                let callback =
                                    hil::Callback::new(process_s.borrow_mut(),
                                                       callback_ptr);
                                match driver {
                                    Some(d) => d.subscribe(process.r1(),
                                                           callback),
                                    None => -1
                                }
                            });
                            process.set_r0(res);
                        },
                        Some(syscall::COMMAND) => {
                            let res = platform.with_driver(process.r0(), |driver| {
                                match driver {
                                    Some(d) => d.command(process.r1(),
                                                         process.r2()),
                                    None => -1
                                }
                            });
                            process.set_r0(res);
                        },
                        _ => {}
                    }
                }
            }

            support::atomic(|| {
                if !platform.has_pending_interrupts() {
                    support::wfi();
                }
            })
        };
    }
}
