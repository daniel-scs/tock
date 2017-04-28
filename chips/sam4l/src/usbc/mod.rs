//! SAM4L USB controller

// See below for how to force linux usb ports to use Full/Low speed instead of High:
//   https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-bus-pci-drivers-ehci_hcd

#![allow(dead_code)]

use nvic;
use kernel::hil;
use pm::{Clock, HSBClock, PBBClock, enable_clock, disable_clock};
use core::fmt;
use core::slice;
use core::ptr;
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
    descriptors: [Endpoint; 8],
    buf: [u8; 8],
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
            buf: [0xab; 8],
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

                        if let Mode::Device{ speed, .. } = mode {
                            UDCON_LS.write(speed)
                        }

                        USBCON_UIMOD.write(mode);   // see registers.rs: maybe wrong bit?
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

    pub fn endpoint_bank_set_buffer(&self, endpoint: EndpointIndex, bank: BankIndex,
                                    buf: *mut u8) {
        let e: usize = From::from(endpoint);
        let b: usize = From::from(bank);

        self.descriptors[e][b].set_addr(buf);
        self.descriptors[e][b].set_packet_size(PacketSize::single(0));
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
                State::Idle(Mode::Device{ ref mut config, .. }) => {
                    *config = Some(cfg);
                }
                State::Active(Mode::Device{ ref mut config, .. }) => {
                    *config = Some(cfg);
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

        // Specify which endpoint interrupts we want, among:
        //      TXIN | RXOUT | RXSTP | NAKOUT | NAKIN |
        //      ERRORF | STALLED | CRCERR | RAMACERR
        endpoint_enable_only_interrupts(endpoint, RXSTP | ERRORF | STALLED | CRCERR | RAMACERR);
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

        let mut state = self.state.take().unwrap_or(State::Reset);

        // Handle host-mode interrupt TODO

        // Handle device-mode interrupt

        let udint: u32 = UDINT.read();

        let p = self.descriptors[0][0].addr.get();
        debug!("--> USB INT: UDINT={:08x}{:?}, B_{}_{} @ {:?}", udint, UdintFlags(udint), 0, 0, p);

        if udint & UDINT_EORST != 0 {
            if let State::Active(Mode::Device{ state: ref mut dstate, .. }) = state {
                *dstate = DeviceState::Init;
            }
            self.state.replace(state);

            // USB bus reset
            self.bus_reset();

            debug_regs();

            // Acknowledge the interrupt
            UDINTCLR.write(UDINT_EORST);

            return;
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

        if let State::Active(Mode::Device{ state: ref mut dstate, .. }) = state {

            for endpoint in 0..9 {
                if udint & (1 << (12 + endpoint)) == 0 {
                    // No interrupts for this endpoint
                    continue;
                }

                let mut status = UESTAn.n(endpoint).read();
                debug!("UESTA{}={:08x}{:?}", endpoint, status, UestaFlags(status));

                // e.g., UESTA0=00021015 means:
                //   CTLDIR=1 (next is IN)
                //   NBUSYBANK=1
                //
                //   NAKINI=1 (NAK was sent)
                //   RXSTPI
                //   TXINI

                if status & RXSTP != 0 {
                    if *dstate == DeviceState::Init {
                        // We received a SETUP transaction
                        debug!("D({}) RXSTP", endpoint);

                        // client.received_setup(bank)
                        self.debug_show_d0();

                        if status & CTRLDIR != 0 {
                            *dstate = DeviceState::SetupIn;

                            // Wait until bank is clear to send
                            // Also, wait for NAKIN to signal end of IN stage
                            UESTAnCLR.n(endpoint).write(NAKIN);
                            status &= !NAKIN;
                            endpoint_enable_interrupts(endpoint, TXIN | NAKIN);
                            // endpoint_disable_interrupts(endpoint, RXSTP);

                            // Don't handle TXIN, if any, until next interrupt (for ease of debugging)
                            status &= !TXIN;
                        }
                        else {
                            *dstate = DeviceState::SetupOut;

                            debug!("-> SetupOut (UNIMPLEMENTED)");
                        }

                        // Acknowledge
                        UESTAnCLR.n(endpoint).write(RXSTP);
                    }
                    else {
                        debug!("** ignoring unexpected RXSTP in dstate {:?}", *dstate);

                        // Acknowledge
                        UESTAnCLR.n(endpoint).write(RXSTP);
                    }
                }

                if status & NAKIN != 0 {
                    if *dstate == DeviceState::SetupIn {
                        // The host has aborted the IN stage
                        debug!("D({}) NAKIN", endpoint);

                        *dstate = DeviceState::SetupOut;

                        // Await end of Status stage
                        endpoint_disable_interrupts(endpoint, TXIN | NAKIN);
                        endpoint_enable_interrupts(endpoint, RXOUT);

                        // Acknowledge
                        UESTAnCLR.n(endpoint).write(NAKIN);
                    }
                }

                if status & TXIN != 0 {
                    if *dstate == DeviceState::SetupIn {
                        // The data bank is ready to receive another IN payload
                        debug!("D({}) TXIN", endpoint);

                        // This appears to be necessary because the address has been changed
                        // somehow (!?!)
                        self.descriptors[0][0].set_addr(&self.buf as *const u8 as *mut u8);

                        // If IN data waiting, bank it for transmission

                        // DEBUG: The first 8 bytes of a standard device descriptor
                        let b = self.descriptors[0][0].addr.get();
                        unsafe {
                            ptr::write_volatile(b.offset(0), 18);   // Length
                            ptr::write_volatile(b.offset(1), 1);    // DEVICE descriptor
                            ptr::write_volatile(b.offset(2), 2); //
                            ptr::write_volatile(b.offset(3), 0); // USB 2.0
                            ptr::write_volatile(b.offset(4), 0); // Class
                            ptr::write_volatile(b.offset(5), 0); // Subclass
                            ptr::write_volatile(b.offset(6), 0); // Protocol
                            ptr::write_volatile(b.offset(7), 8); // Max packet size
                        }
                        self.descriptors[0][0].packet_size.set(PacketSize::single(8));

                        self.debug_show_d0();

                        // Signal to the controller that the IN payload is ready to send
                        UESTAnCLR.n(endpoint).write(TXIN);

                        // (Continue awaiting TXIN and NAKIN until end of IN stage)
                    }
                    else {
                        // Nothing to send: ignore
                    }

                    // For non-control endpoints:
                    // clear FIFOCON to allow send
                }

                if status & RXOUT != 0 {
                    if *dstate == DeviceState::SetupOut {
                        debug!("D({}) RXOUT", endpoint);
                        // self.debug_show_d0();

                        *dstate = DeviceState::Init;

                        // Wait for next SETUP
                        endpoint_disable_interrupts(endpoint, RXOUT);
                        endpoint_enable_interrupts(endpoint, RXSTP);

                        // Acknowledge
                        UESTAnCLR.n(endpoint).write(RXOUT);
                    }

                    // For non-control endpoints:
                    // clear FIFOCON to free bank

                    // client.received_out(bank)
                }

                if status & NAKOUT != 0 {
                    debug!("D({}) NAKOUT", endpoint);

                    // Acknowledge
                    UESTAnCLR.n(endpoint).write(NAKOUT);
                }

                if status & STALLED != 0 {
                    debug!("D({}) STALLED/CRCERR", endpoint);

                    // Acknowledge
                    UESTAnCLR.n(endpoint).write(STALLED);
                }

                if status & RAMACERR != 0 {
                    debug!("D({}) RAMACERR", endpoint);

                    // Acknowledge
                    UESTAnCLR.n(endpoint).write(RAMACERR);
                }
            }

        } // if Device

        self.state.replace(state);
    }

    fn debug_show_d0(&self) {
        for bi in 0..1 {
            let b = &self.descriptors[0][bi];
            let addr = b.addr.get();
            let buf = if addr.is_null() { None }
                      else { unsafe { Some(slice::from_raw_parts(addr, 8)) } };

            debug!("B_0_{} at addr {:?}: \
                   \n     {:?}\
                   \n     {:?}\
                   \n     {:?}",
                   bi, b.addr.get(), b.packet_size.get(), b.ctrl_status.get(),
                   buf);
        }
    }

    fn bus_reset(&mut self) {
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

        // XXX DEBUG
        self.descriptors[0][0].set_addr(&self.buf as *const u8 as *mut u8);
        self.descriptors[0][0].set_packet_size(PacketSize::single(0));

        // alert client?
    }

    fn reconfigure(&mut self) {
        if let Some(Mode::Device{ speed, config, .. }) = self.mode() {
            UDCON_LS.write(speed);

            if let Some(config) = config {
                self.endpoint_configure(0, config);
            }
        }
    }

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
                Mode::Device{ speed, .. } => Some(speed),
                Mode::Host => {
                    None // XXX USBSTA.SPEED
                }
            },
            _ => None
        }
    }

    // Remote wakeup (Device -> Host, after receiving DEVICE_REMOTE_WAKEUP)
}

#[inline]
fn endpoint_disable_interrupts(endpoint: u32, mask: u32) {
    UECONnCLR.n(endpoint).write(mask);
}

#[inline]
fn endpoint_enable_interrupts(endpoint: u32, mask: u32) {
    UECONnSET.n(endpoint).write(mask);
}

#[inline]
fn endpoint_enable_only_interrupts(endpoint: u32, mask: u32) {
    endpoint_disable_interrupts(endpoint, !0);
    endpoint_enable_interrupts(endpoint, mask);
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

struct UdintFlags(u32);

impl fmt::Debug for UdintFlags {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: u32 = self.0;

        write!(f, "{{");
        if w & UDINT_WAKEUP != 0 {
            write!(f, "w");
        }
        if w & UDINT_SOF != 0 {
            write!(f, "s");
        }

        if w & UDINT_SUSP != 0 {
            write!(f, " SUSP");
        }
        if w & UDINT_EORST != 0 {
            write!(f, " EORST");
        }
        if w & UDINT_EORSM != 0 {
            write!(f, " EORSM");
        }
        if w & UDINT_UPRSM != 0 {
            write!(f, " UPRSM");
        }

        for i in 0..9 {
            if w & (1 << (12 + i)) != 0 {
                write!(f, " EP{}", i);
            }
        }
        write!(f, "}}")
    }
}

struct UestaFlags(u32);

impl fmt::Debug for UestaFlags {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: u32 = self.0;

        write!(f, "{{");
        if w & TXIN != 0 {
            write!(f, "TXIN ");
        }
        if w & RXOUT != 0 {
            write!(f, "RXOUT ");
        }
        if w & RXSTP != 0 {
            write!(f, "RXSTP");
        }
        if w & ERRORF != 0 {
            write!(f, "/ERRORF ");
        }
        if w & NAKOUT != 0 {
            write!(f, "NAKOUT ");
        }
        if w & NAKIN != 0 {
            write!(f, "NAKIN ");
        }
        if w & STALLED != 0 {
            write!(f, "STALLED ");
        }
        if w & CRCERR != 0 {
            write!(f, "CRCERR");
        }
        if w & RAMACERR != 0 {
            write!(f, "/RAMACERR ");
        }
        write!(f, "NBUSYBK={} ", (w >> 12) & 0x3);
        write!(f, "CURBK={} ", (w >> 14) & 0x3);
        write!(f, "CTRLDIR={}", if w & CTRLDIR != 0 { "IN" } else { "OUT" });
        write!(f, "}}")
    }
}

/// Static state to manage the USBC
pub static mut USBC: Usbc<'static> = Usbc::new();

interrupt_handler!(usbc_handler, USBC);
