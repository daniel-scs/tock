//! SAM4L USB controller

#![allow(dead_code)]

use nvic;
use kernel::hil;
use pm::{Clock, HSBClock, PBBClock, enable_clock, disable_clock};
use core::cell::Cell;
use scif;

mod data;
use self::data::*;

mod common_register;
#[macro_use]
mod register_macros;
mod registers;
use self::registers::*;

macro_rules! client_err {
    [ $offset:expr ] => {
        { /* ignore error */ }
    };
}

/// State for managing the USB controller
pub struct Usbc<'a> {
    client: Option<&'a hil::usb::Client>,
    state: Cell<State>,
}

impl<'a> Usbc<'a> {
    pub const fn new() -> Self {
        Usbc {
            client: None,
            state: Cell::new(State::Reset),
        }
    }

    /// Enable the controller's clocks and interrupt and transition to Idle state
    /// (No effect if current state is not Reset)
    pub fn enable(&self, mode: Mode) {
        match self.state.get() {
            State::Reset => {
                unsafe {
                    /* XXX "To follow the usb data rate at 12Mbit/s in full-speed mode, the
                     * CLK_USBC_AHB clock should be at minimum 12MHz."
                     */

                    // Are the USBC clocks enabled at reset?
                    //   10.7.4 says no, but 17.5.3 says yes
                    // Also, "Being in Idle state does not require the USB clocks to be activated"
                    //   (17.6.2)
                    enable_clock(Clock::HSB(HSBClock::USBC));
                    enable_clock(Clock::PBB(PBBClock::USBC));

                    /* XXX "The 48MHz USB clock is generated by a dedicated generic clock from the
                     * SCIF module. Before using the USB, the user must ensure that the USB generic
                     * clock (GCLK_USBC) is enabled at 48MHz in the SCIF module."
                     *
                     * Generic clock 7 is allocated to the USBC (13.8)
                     */
                    // scif::generic_clock_enable(scif::GenericClock::GCLK7, scif::ClockSource::XXX);

                    nvic::disable(nvic::NvicIdx::USBC);
                    nvic::clear_pending(nvic::NvicIdx::USBC);
                    nvic::enable(nvic::NvicIdx::USBC);

                    // If we got to this state via disable() instead of chip reset,
                    // the values USBCON.FRZCLK, USBCON.UIMOD, UDCON.LS have *not* been reset to
                    // their default values.

                    if let Mode::Device(speed) = mode {
                        UDCON_LS.write(speed)
                    }
                    USBCON_UIMOD.write(mode);
                    USBCON_FRZCLK.write(false);
                    USBCON_USBE.write(true);
                }
                self.state.set(State::Idle(mode));
            }
            _ => { /* Already enabled */ }
        }
    }

    /// Attach to the USB bus
    pub fn attach(&self) {
        match self.state.get() {
            State::Reset => {
                client_err!("Not enabled");
            }
            State::Active(_) => {
                client_err!("Already attached");
            }
            State::Idle(mode) => {
                UDCON_DETACH.write(false);
                self.state.set(State::Active(mode));
            }
        }
    }

    /// Disable the controller, its interrupt, and its clocks
    pub fn disable(&self) {
        if self.state.get() != State::Reset {
            unsafe {
                USBCON_USBE.write(false);

                nvic::disable(nvic::NvicIdx::USBC);

                disable_clock(Clock::PBB(PBBClock::USBC));
                disable_clock(Clock::HSB(HSBClock::USBC));
            }
            self.state.set(State::Reset);
        }
    }

    /// Set address
    pub fn set_address(&self, _addr: Address) {
        /*
        if self.address == 0 && addr != 0 {
            self.start_transaction(Tx::Setup(Request::new(SET_ADDRESS(addr))));
            // UDCON.UADD.set(addr);
            // UDCON.ADDEN.clear();
            self.send(self.control_endpoint(), In::new(empty()));
            // UDCON.ADDEN.set();
        }
        */
    }

    /// Set a client to receive data from the USBC
    pub fn set_client(&mut self, client: &'a hil::usb::Client) {
        self.client = Some(client);
    }

    /// Get the client
    pub fn get_client(&self) -> Option<&'a hil::usb::Client> {
        self.client
    }

    /// Handle an interrupt from the USBC
    pub fn handle_interrupt(&mut self) {
        // UDINT.SUSP => goto (Idle ==? Suspend)
        // "To further reduce power consumption it is recommended to freeze the USB clock by
        // writing a one to the Freeze USB Clock (FRZCLK) bit in USBCON when the USB bus is in
        // suspend mode.
        //
        // To recover from the suspend mode, the user shall wait for the Wakeup (WAKEUP) interrupt
        // bit, which is set when a non-idle event is detected, and then write a zero to FRZCLK.
        //
        // As the WAKEUP interrupt bit in UDINT is set when a non-idle event is detected, it can
        // occur regardless of whether the controller is in the suspend mode or not."

        // WAKEUP => goto Active
        // UDINT.EORST => USB reset
    }

    pub fn mode(&self) -> Option<Mode> {
        match self.state.get() {
            State::Idle(mode) => Some(mode),
            _ => None
        }
    }

    pub fn speed(&self) -> Option<Speed> {
        match self.mode() {
            Some(mode) => match mode {
                Mode::Device(speed) => Some(speed),
                Mode::Host => {
                    None // XXX USBSTA.SPEED
                }
            },
            _ => None
        }
    }

    // Remote wakeup (Device -> Host, after receiving DEVICE_REMOTE_WAKEUP)
}

/// Static state to manage the USBC
pub static mut USBC: Usbc<'static> = Usbc::new();

interrupt_handler!(usbc_handler, USBC);
