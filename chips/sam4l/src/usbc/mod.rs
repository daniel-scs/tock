//! SAM4L USB controller

#![allow(dead_code)]

use nvic;
use kernel::hil;
use pm::{Clock, HSBClock, PBBClock, enable_clock, disable_clock};
use core::cell::Cell;
use scif;

pub mod data;
use self::data::*;

mod common_register;
#[macro_use]
mod register_macros;
mod registers;
use self::registers::*;

macro_rules! client_err {
    [ $msg:expr ] => {
        debug!($msg)
    };
}

/// State for managing the USB controller
pub struct Usbc<'a> {
    client: Option<&'a hil::usb::Client>,
    state: Cell<State>,
    pub descriptors: [Endpoint; 8],
}

impl<'a> Usbc<'a> {
    const fn new() -> Self {
        Usbc {
            client: None,
            state: Cell::new(State::Reset),
            descriptors: [ new_endpoint(),
                           new_endpoint(),
                           new_endpoint(),
                           new_endpoint(),
                           new_endpoint(),
                           new_endpoint(),
                           new_endpoint(),
                           new_endpoint() ],
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

                    UDESC.write(&self.descriptors as *const Endpoint as u32);

                    // Device interrupts
                    let udints = UDINT_SUSP |
                                 UDINT_SOF |
                                 UDINT_EORST |
                                 UDINT_WAKEUP |
                                 UDINT_EORSM |
                                 UDINT_UPRSM;

                    // Clear all pending device global interrupts
                    UDINTCLR.write(udints);

                    // Enable all device global interrupts
                    UDINTESET.write(udints);

                    /*
                    // Host interrupts
                    let uhints = 0x7f;

                    // Clear all pending host global interrupts
                    UHINTCLR.write(uhints);

                    // Enable all host global interrupts
                    UHINTESET.write(uhints);
                    */

                    debug!("Enabled.");
                    debug_regs();
                }
                self.state.set(State::Idle(mode));
            }
            _ => {
                client_err!("Already enabled")
            }
        }
    }

    /// Attach to the USB bus after enabling USB clock
    pub fn attach(&self) {
        match self.state.get() {
            State::Reset => {
                client_err!("Not enabled");
            }
            State::Active(_) => {
                client_err!("Already attached");
            }
            State::Idle(mode) => {
                // XXX: This setting works only because the imix configures DFLL0 to
                // produce 48MHz
                scif::generic_clock_enable(scif::GenericClock::GCLK7, scif::ClockSource::DFLL0);
                debug!("Waiting for USB clock ...");
                while !USBSTA_CLKUSABLE.read() {}
                debug!("USB clock ready."); 

                UDCON_DETACH.write(false);
                debug!("Attached.");
                debug_regs();
                self.state.set(State::Active(mode));
            }
        }
    }

    pub fn stimulate_interrupts(&self) {
        UDINTSET.write(UDINT_WAKEUP | UDINT_EORST | UDINT_SOF);
        // UHINTSET.write(0x7f);
    }

    /// Detach from the USB bus.  Also disable USB clock to save energy.
    pub fn detach(&self) {
        match self.state.get() {
            State::Reset => {
                client_err!("Not enabled");
            }
            State::Idle(_) => {
                client_err!("Not attached");
            }
            State::Active(mode) => {
                UDCON_DETACH.write(true);

                scif::generic_clock_disable(scif::GenericClock::GCLK7);

                self.state.set(State::Idle(mode));
            }
        }
    }

    /// Disable the controller, its interrupt, and its clocks
    pub fn disable(&self) {
        if let State::Active(_) = self.state.get() {
            self.detach();
        }

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
    pub fn set_address(&self /* , _addr: Address */) {
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

    /// Configure and enable an endpoint
    pub fn enable_endpoint(&self, endpoint: u32, cfg: EndpointConfig) {
		/*
		Before using an endpoint, the user should setup the endpoint address for each bank. Depending
		on the direction, the type, and the packet-mode (single or multi-packet), the user should also ini-
		tialize the endpoint packet size, and the endpoint control and status fields, so that the USBC
		controller does not compute random values from the RAM.
		*/

        // Enable the endpoint (meaning the controller will respond to requests)
        UERST.set_bit(endpoint);

        // Configure the endpoint
        UECFGn.n(endpoint).write(cfg);

        // Specify which endpoint interrupts we want
        UECONnSET.n(endpoint).write(TXIN | RXOUT | RXSTP | ERRORF | NAKOUT |
                                    NAKIN | STALLED | CRCERR | RAMACERR);

        // Set EPnINTE (n == endpoint), enabling interrupts for this endpoint
        UDINTESET.set_bit(12 + endpoint);

        debug!("Enabled endpoint {}", endpoint);
        debug_regs();
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
        debug!("USB interrupt!");

        // Handle host-mode interrupt
        // XXX TODO

        // Handle device-mode interrupt

        let udint: u32 = UDINT.read();

        if udint & 1 != 0 {
            debug!("UDINT SUSP");

            // goto (Idle ==? Suspend)
            //
            // "To further reduce power consumption it is recommended to freeze the USB clock by
            // writing a one to the Freeze USB Clock (FRZCLK) bit in USBCON when the USB bus is in
            // suspend mode.
        }

        if udint & (1 << 2) != 0 {
            debug!("UDINT SOF");
        }

        if udint & (1 << 3) != 0 {
            // USB bus reset
            debug!("UDINT EORST");
            self.state.set(State::Reset);
            // alert client?
        }

        if udint & (1 << 4) != 0 {
            debug!("UDINT WAKEUP");

            // goto Active
            //
            // To recover from the suspend mode, the user shall wait for the Wakeup (WAKEUP) interrupt
            // bit, which is set when a non-idle event is detected, and then write a zero to FRZCLK.
            //
            // As the WAKEUP interrupt bit in UDINT is set when a non-idle event is detected, it can
            // occur regardless of whether the controller is in the suspend mode or not."
        }

        if udint & (1 << 5) != 0 {
            // End of resume
            debug!("UDINT EORSM");
        }

        if udint & (1 << 6) != 0 {
            debug!("UDINT UPRSM");
        }

        for endpoint in 0..9 {
            if udint & (1 << (12 + endpoint)) == 0 {
                // No interrupts for this endpoint
                continue;
            }

            let status = UESTAn.n(endpoint).read();

            if status & TXIN != 0 {
                debug!("D({}) TXINI", endpoint);
                // if outbound data waiting, bank it for transmission
                // clear TXINI
            }

            if status & RXOUT != 0 {
                debug!("D({}) RXOUTI", endpoint);
                // client.received_out(bank)
                // clear RXOUTI
            }

            if status & RXSTP != 0 {
                debug!("D({}) RXSTPI/ERRORFI", endpoint);
                // check error?
                // client.received_setup(bank)
                // clear RXSTPI
                // UESTAnCLR(endpoint).write(1 << 2);
            }

            if status & NAKOUT != 0 {
                debug!("D({}) NAKOUTI", endpoint);
            }

            if status & NAKIN != 0 {
                debug!("D({}) NAKINI", endpoint);
            }

            if status & STALLED != 0 {
                debug!("D({}) STALLEDI/CRCERRI", endpoint);
            }
        }
    }

    pub fn state(&self) -> State {
        self.state.get()
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

fn debug_regs() {
    debug!("    USBFSM={:08x}", USBFSM.read());
    debug!("    USBCON={:08x}", USBCON.read());
    debug!("    USBSTA={:08x}", USBSTA.read());
    debug!("     UDCON={:08x}", UDCON.read());
    debug!("    UDINTE={:08x}", UDINTE.read());
    debug!("     UDINT={:08x}", UDINT.read());
    debug!("     UERST={:08x}", UERST.read());
    debug!("    UECFG0={:08x}", UECFG0.read_word());
    debug!("    UECON0={:08x}", UECON0.read());
    // debug!("    UHINTE={:08x}", UHINTE.read());
    // debug!("     UHINT={:08x}", UDINT.read());
}

/// Static state to manage the USBC
pub static mut USBC: Usbc<'static> = Usbc::new();

interrupt_handler!(usbc_handler, USBC);
