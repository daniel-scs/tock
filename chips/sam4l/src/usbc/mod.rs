//! SAM4L USB controller

// See below for how to force linux usb ports to use Full/Low speed instead of High:
//   https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-bus-pci-drivers-ehci_hcd

#![allow(dead_code)]

use nvic;
use kernel::hil;
use pm::{Clock, HSBClock, PBBClock, enable_clock, disable_clock};
use core::slice;
use scif;
use kernel::common::take_cell::MapCell;

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
    state: MapCell<State>,
    pub descriptors: [Endpoint; 8],
}

impl<'a> Usbc<'a> {
    const fn new() -> Self {
        Usbc {
            client: None,
            state: MapCell::new(State::Reset),
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
        self.state.map(|state| {
            match *state {
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

                        if let Mode::Device(speed, _) = mode {
                            UDCON_LS.write(speed)
                        }

                        USBCON_UIMOD.write(mode);
                        USBCON_FRZCLK.write(false);
                        USBCON_USBE.write(true);

                        UDESC.write(&self.descriptors as *const Endpoint as u32);

                        // Device interrupts
                        let udints = // UDINT_SUSP | // XXX ignore while debugging interrupts
                                     // UDINT_SOF |
                                     UDINT_EORST |
                                     UDINT_EORSM |
                                     UDINT_UPRSM;

                        // Clear pending device global interrupts
                        UDINTCLR.write(udints);

                        // Enable device global interrupts
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
                        // debug_regs();
                    }
                    *state = State::Idle(mode);
                }
                _ => {
                    client_err!("Already enabled")
                }
            }
        });
    }

    /// Attach to the USB bus after enabling USB clock
    pub fn attach(&self) {
        self.state.map(|state| {
            match *state {
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
                    // debug!("Waiting for USB clock ...");
                    while !USBSTA_CLKUSABLE.read() {}
                    // debug!("USB clock ready."); 

                    UDCON_DETACH.write(false);
                    debug!("Attached.");
                    debug_regs();
                    *state = State::Active(mode);
                }
            }
        });
    }

    pub fn stimulate_interrupts(&self) {
        UDINTSET.write(UDINT_WAKEUP | UDINT_EORST | UDINT_SOF);
        // UHINTSET.write(0x7f);
    }

    /// Detach from the USB bus.  Also disable USB clock to save energy.
    pub fn detach(&self) {
        self.state.map(|state| {
            match *state {
                State::Reset => {
                    client_err!("Not enabled");
                }
                State::Idle(_) => {
                    client_err!("Not attached");
                }
                State::Active(mode) => {
                    UDCON_DETACH.write(true);

                    scif::generic_clock_disable(scif::GenericClock::GCLK7);

                    *state = State::Idle(mode);
                }
            }
        });
    }

    /// Disable the controller, its interrupt, and its clocks
    pub fn disable(&self) {
        if self.state.map_or(false, |state| { if let State::Active(_) = *state { true } else { false } }) {
            self.detach();
        }

        self.state.map(|state| {
            if *state != State::Reset {
                unsafe {
                    USBCON_USBE.write(false);

                    nvic::disable(nvic::NvicIdx::USBC);

                    disable_clock(Clock::PBB(PBBClock::USBC));
                    disable_clock(Clock::HSB(HSBClock::USBC));
                }
                *state = State::Reset;
            }
        });
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
    /// XXX: include addr and packetsize?
    pub fn endpoint_enable(&self, endpoint: u32, cfg: EndpointConfig) {
        self.state.map(|state| {

            // Record config in case of later reset
            match *state {
                State::Reset => {
                    client_err!("Not enabled");
                }
                State::Idle(Mode::Device(_, ref mut cfgp)) => {
                    *cfgp = Some(cfg);
                }
                State::Active(Mode::Device(_, ref mut cfgp)) => {
                    *cfgp = Some(cfg);
                }
                _ => {
                    client_err!("Not in Device mode");
                }
            }
        });

		/* XXX
		Before using an endpoint, the user should setup the endpoint address for each bank. Depending
		on the direction, the type, and the packet-mode (single or multi-packet), the user should also ini-
		tialize the endpoint packet size, and the endpoint control and status fields, so that the USBC
		controller does not compute random values from the RAM.
		*/

        // Enable the endpoint (meaning the controller will respond to requests)
        UERST.set_bit(endpoint);

        self.endpoint_configure(endpoint, cfg);

        // Set EPnINTE, enabling interrupts for this endpoint
        UDINTESET.set_bit(12 + endpoint);

        debug!("Enabled endpoint {}", endpoint);
        // debug_regs();
    }

    fn endpoint_configure(&self, endpoint: u32, cfg: EndpointConfig) {
        // Configure the endpoint
        UECFGn.n(endpoint).write(cfg);

        // Specify which endpoint interrupts we want
        // UECONnSET.n(endpoint).write(TXIN | RXOUT | RXSTP | ERRORF | NAKOUT |
        //                             NAKIN | STALLED | CRCERR | RAMACERR);
        UECONnSET.n(endpoint).write(RXSTP | RXOUT | TXIN |
                                    ERRORF | STALLED | CRCERR | RAMACERR);
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

        // Handle host-mode interrupt
        // XXX TODO

        // Handle device-mode interrupt

        let udint: u32 = UDINT.read();

        // debug!("USB interrupt! UDINT={:08x}", udint);

        if udint & UDINT_EORST != 0 {
            // USB bus reset
            self.reset();

            // Acknowledge the interrupt
            UDINTCLR.write(UDINT_EORST);
        }

        if udint & UDINT_SUSP != 0 {
            // The transceiver has been suspended due to the bus being idle for 3ms.
            // This condition is over when WAKEUP is set.

            // "To further reduce power consumption it is recommended to freeze the USB clock by
            // writing a one to the Freeze USB Clock (FRZCLK) bit in USBCON when the USB bus is in
            // suspend mode.
            //
            // To recover from the suspend mode, the user shall wait for the Wakeup (WAKEUP) interrupt
            // bit, which is set when a non-idle event is detected, and then write a zero to FRZCLK.
            //
            // As the WAKEUP interrupt bit in UDINT is set when a non-idle event is detected, it can
            // occur regardless of whether the controller is in the suspend mode or not."

            // Subscribe to WAKEUP
            UDINTSET.write(UDINT_WAKEUP);

            // Acknowledge the "suspend" event
            UDINTCLR.write(UDINT_SUSP);
        }

        if udint & UDINT_WAKEUP != 0 {
            // Unfreeze the clock (and unsleep the MCU)

            // Unsubscribe from WAKEUP
            UDINTECLR.write(UDINT_WAKEUP);

            // Acknowledge the interrupt
            UDINTCLR.write(UDINT_WAKEUP);
        }

        if udint & UDINT_SOF != 0 {
            // Start of frame

            // Acknowledge the interrupt
            UDINTCLR.write(UDINT_SOF);
        }

        if udint & UDINT_EORSM != 0 {
            // End of resume
            debug!("UDINT EORSM");
        }

        if udint & UDINT_UPRSM != 0 {
            debug!("UDINT UPRSM");
        }

        for endpoint in 0..9 {
            if udint & (1 << (12 + endpoint)) == 0 {
                // No interrupts for this endpoint
                continue;
            }

            let status = UESTAn.n(endpoint).read();
            debug!("UESTA{}={:08x}", endpoint, status);

            // UESTA0=00021015
            //   CTLDIR=1 (next is IN)
            //   NBUSYBANK=1
            //
            //   NAKINI=1 (NAK was sent)
            //   RXSTPI
            //   TXINI
            // D(0) TXINI
            // D(0) RXSTPI

            if status & RXSTP != 0 {
                debug!("D({}) RXSTPI/ERRORFI", endpoint);
                // client.received_setup(bank)
                self.debug_show_d0();

                // Acknowledge
                UESTAnCLR.n(endpoint).write(RXSTP);
            }

            if status & TXIN != 0 {
                debug!("D({}) TXINI", endpoint);

                // If outbound data waiting, bank it for transmission
                self.descriptors[0][0].packet_size.set(PacketSize::single(0));
                
                // Signal to the controller that the OUT payload is ready to send
                UESTAnCLR.n(endpoint).write(TXIN);

                // For non-control endpoints:
                // clear FIFOCON to allow send
            }

            if status & RXOUT != 0 {
                debug!("D({}) RXOUTI", endpoint);
                // client.received_out(bank)
                self.debug_show_d0();

                // Acknowledge
                UESTAnCLR.n(endpoint).write(RXOUT);

                // For non-control endpoints:
                // clear FIFOCON to free bank
            }

            if status & NAKOUT != 0 {
                debug!("D({}) NAKOUTI", endpoint);

                // Acknowledge
                UESTAnCLR.n(endpoint).write(NAKOUT);
            }

            if status & NAKIN != 0 {
                debug!("D({}) NAKINI", endpoint);

                // Acknowledge
                UESTAnCLR.n(endpoint).write(NAKIN);
            }

            if status & STALLED != 0 {
                debug!("D({}) STALLEDI/CRCERRI", endpoint);

                // Acknowledge
                UESTAnCLR.n(endpoint).write(STALLED);
            }
        }

        // debug!("Handled interrupt");
    }

    fn debug_show_d0(&self) {
        for bi in 0..2 {
            let b = &self.descriptors[0][bi];

            debug!("B_0_{}: \
                   \n     packet_size={:?}\
                   \n     control_status={:?}",
                   bi, b.packet_size.get(), b.ctrl_status.get());

            let addr = b.addr.get().0;
            if bi == 0 && addr != 0 {
                if b.packet_size.get().byte_count() == 8 {
                    let buf: &[u8] = unsafe { slice::from_raw_parts(addr as *const u8, 8) };
                    debug!("B_0_{}: \
                           \n     {:?}", bi, buf);
                }
            }
        }
    }

    fn reset(&mut self) {
        debug!("USB Bus Reset");

        let must_reconfigure = self.state.map_or(false, |state| {
            match *state {
                State::Reset => {
                    /* Ignore */
                    false
                }
                State::Idle(_) => {
                    // XXX does this ever happen?
                    true
                }
                State::Active(_) => {
                    true
                }
            }
        });
        if must_reconfigure {
            self.reconfigure();
        }

        // alert client?
    }

    fn reconfigure(&mut self) {
        if let Some(Mode::Device(speed, cfg)) = self.mode() {
            UDCON_LS.write(speed);

            if let Some(cfg) = cfg {
                self.endpoint_configure(0, cfg);
            }
        }
    }

    /*
    pub fn state(&self) -> State {
        self.state.get()
    }
    */

    pub fn mode(&self) -> Option<Mode> {
        self.state.map_or(None, |state| {
            match *state {
                State::Idle(mode) => Some(mode),
                State::Active(mode) => Some(mode),
                _ => None
            }
        })
    }

    pub fn speed(&self) -> Option<Speed> {
        match self.mode() {
            Some(mode) => match mode {
                Mode::Device(speed, _) => Some(speed),
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
    debug!("    registers:\
            \n    USBFSM={:08x}\
            \n    USBCON={:08x}\
            \n    USBSTA={:08x}\
            \n     UDCON={:08x}\
            \n    UDINTE={:08x}\
            \n     UDINT={:08x}\
            \n     UERST={:08x}\
            \n    UECFG0={:08x}\
            \n    UECON0={:08x}",

           USBFSM.read(),
           USBCON.read(),
           USBSTA.read(),
           UDCON.read(),
           UDINTE.read(),
           UDINT.read(),
           UERST.read(),
           UECFG0.read_word(),
           UECON0.read());

    // debug!("    UHINTE={:08x}", UHINTE.read());
    // debug!("     UHINT={:08x}", UDINT.read());
}

/// Static state to manage the USBC
pub static mut USBC: Usbc<'static> = Usbc::new();

interrupt_handler!(usbc_handler, USBC);
