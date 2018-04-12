//! SAM4L USB controller

pub mod data;

#[macro_use]
mod common;
mod registers;

use self::common::register::*;
use self::data::*;
use self::registers::*;
use core::cell::Cell;
use core::fmt;
use core::slice;
use kernel::common::VolatileCell;
use kernel::common::take_cell::MapCell;
use kernel::hil;
use kernel::hil::usb::*;
use pm;
use pm::{disable_clock, enable_clock, Clock, HSBClock, PBBClock};
use scif;

macro_rules! client_err {
    [ $msg:expr ] => {
        debug!($msg)
    };
}

struct UsbcRegisters {
    udcon: ReadWrite<u32, DeviceControl>,
    udint: ReadOnly<u32, DeviceInterrupt>,
    udintclr: WriteOnly<u32, DeviceInterrupt>,
    udintset: WriteOnly<u32, DeviceInterrupt>,
    udinte: ReadOnly<u32, DeviceInterrupt>,
    udinteclr: WriteOnly<u32, DeviceInterrupt>,
    udinteset: WriteOnly<u32, DeviceInterrupt>,
    uerst: ReadWrite<u32>,
    udfnum: ReadOnly<u32>,
    _reserved0: [u8; 0xdc], // 220 bytes
    // 0x100
    uecfg: [ReadWrite<u32>; 12],
    uesta: [ReadOnly<u32, EndpointStatus>; 12],
    uestaclr: [WriteOnly<u32, EndpointStatus>; 12],
    uestaset: [WriteOnly<u32, EndpointStatus>; 12],
    uecon: [ReadOnly<u32>; 12],
    ueconset: [WriteOnly<u32>; 12],
    ueconclr: [WriteOnly<u32>; 12],
    _reserved1: [u8; 0x1b0], // 432 bytes
    // 0x400 = 1024
    uhcon: ReadWrite<u32>,
    uhint: ReadOnly<u32>,
    uhintclr: WriteOnly<u32>,
    uhintset: WriteOnly<u32>,
    uhinte: ReadOnly<u32>,
    uhinteclr: WriteOnly<u32>,
    uhinteset: WriteOnly<u32>,
    uprst: ReadWrite<u32>,
    uhfnum: ReadWrite<u32>,
    uhsofc: ReadWrite<u32>,
    _reserved2: [u8; 0xd8], // 216 bytes
    // 0x500 = 1280
    upcfg: [ReadWrite<u32>; 12],
    upsta: [ReadOnly<u32>; 12],
    upstaclr: [WriteOnly<u32>; 12],
    upstaset: [WriteOnly<u32>; 12],
    upcon: [ReadOnly<u32>; 12],
    upconset: [WriteOnly<u32>; 12],
    upconclr: [WriteOnly<u32>; 12],
    upinrq: [ReadWrite<u32>; 12],
    _reserved3: [u8; 0x180], // 384 bytes
    // 0x800 = 2048
    usbcon: ReadWrite<u32, Control>,
    usbsta: ReadOnly<u32>,
    usbstaclr: WriteOnly<u32>,
    usbstaset: WriteOnly<u32>,
    _reserved4: [u8; 8],
    // 0x818
    uvers: ReadOnly<u32>,
    ufeatures: ReadOnly<u32>,
    uaddrsize: ReadOnly<u32>,
    uname1: ReadOnly<u32>,
    uname2: ReadOnly<u32>,
    usbfsm: ReadOnly<u32>,
    udesc: ReadWrite<u32>,
}

register_bitfields![u32,
    Control [
        UIMOD 25 [
            HostMode = 0,
            DeviceMode = 1,
        ],
        USBE 15,
        FRZCLK 14
    ],
    Status [
        SUSPEND OFFSET(16) NUMBITS(1),
        CLKUSABLE OFFSET(14) NUMBITS(1),
        SPEED OFFSET(12) NUMBITS(2) [
            SpeedFull = 0b00,
            SpeedLow = 0b10
        ],
        VBUSRQ OFFSET(9) NUMBITS(1)
    ]
    DeviceControl [
        GNAK OFFSET(17) NUMBITS(1),
        LS OFFSET(12) NUMBITS(1) [
            FullSpeed = 0,
            LowSpeed = 1,
        ],
        RMWKUP OFFSET(9) NUMBITS(1),
        DETACH OFFSET(8) NUMBITS(1),
        ADDEN OFFSET(7) NUMBITS(1),
        UADD OFFSET(0) NUMBITS(7)
    ],
    DeviceInterrupt [
        EPINT OFFSET(12) NUMBITS(8),
        UPRSM OFFSET(6) NUMBITS(1),
        EORSM OFFSET(5) NUMBITS(1),
        WAKEUP OFFSET(4) NUMBITS(1),
        EORST OFFSET(3) NUMBITS(1),
        SOF OFFSET(2) NUMBITS(1),
        SUSP OFFSET(0) NUMBITS(1),
    ],
    EndpointStatus [
        CTRLDIR OFFSET(17) NUMBITS(1) [
            Out = 0,
            In = 1
        ],
        CURRBK OFFSET(14) NUMBITS(2) [
            Bank0 = 0,
            Bank1 = 1
        ],
        NBUSYBK OFFSET(12) NUMBITS(2),
        RAMACER OFFSET(11) NUMBITS(1),
        DTSEQ OFFSET(8) NUMBITS(2) [
            Data0 = 0,
            Data1 = 1,
        ],
        STALLED OFFSET(6) NUMBITS(1),
        CRCERR OFFSET(6) NUMBITS(1),
        NAKIN OFFSET(4) NUMBITS(1),
        NAKOUT OFFSET(3) NUMBITS(1),
        ERRORF OFFSET(2) NUMBITS(1),
        RXSTP OFFSET(2) NUMBITS(1),
        RXOUT OFFSET(1) NUMBITS(1),
        TXIN OFFSET(0) NUMBITS(0)
    ],
    EndpointControl [
        BUSY1E 25,
        BUSY0E 24,
        STALLRQ 19,
        RSTDT 18,
        FIFOCON 14,
        KILLBK 13,
        NBUSYBKE 12,
        RAMACERE 11,
        NREPLY 8,
        STALLEDE 6,
        CRCERRE 6,
        NAKINE 4,
        NAKOUTE 3,
        RXSTPE 2,
        ERRORFE 2,
        RXOUTE 1,
        TXINE 0
    ]
];

const USBC_BASE: u32 = 0x400A5000;

const USBC_REGS = unsafe {
    USBC_BASE as *const UsbcRegisters as &UsbcRegisters
};

/// State for managing the USB controller
// This ensures the `descriptors` field is laid out first
#[repr(C)]
// This provides the required alignment for the `descriptors` field
#[repr(align(8))]
pub struct Usbc<'a> {
    descriptors: [Endpoint; 8],
    client: Option<&'a hil::usb::Client>,
    state: MapCell<State>,
}

#[derive(Default)]
pub struct EP {
    pub index: u32,
}

impl<'a> UsbController for Usbc<'a> {
    type EndpointState = Cell<Option<EP>>;

    fn attach(&self) {
        self._attach();
    }

    fn enable_device(&self, full_speed: bool) {
        let speed = if full_speed { Speed::Full } else { Speed::Low };
        self._enable(Mode::device_at_speed(speed));
    }

    fn endpoint_configure(&self, e: &'static Self::EndpointState, index: u32) {
        e.set(Some(EP { index: index }));
    }

    fn endpoint_set_buffer<'b>(&'b self, e: u32, buf: &[VolatileCell<u8>]) {
        if buf.len() != 8 {
            panic!("Bad endpoint buffer size");
        }
        self.endpoint_bank_set_buffer(EndpointIndex::new(e), BankIndex::Bank0, buf);
    }

    fn endpoint_ctrl_out_enable(&self, e: u32) {
        let cfg = EndpointConfig::new(
            BankCount::Single,
            EndpointSize::Bytes8,
            EndpointDirection::Out,
            EndpointType::Control,
            EndpointIndex::new(e),
        );
        self.endpoint_enable(e, cfg);
    }

    fn set_address(&self, addr: u16) {
        USBC_REGS.udcon.modify(DeviceControl::UADD::val(addr));

        debug!("Set Address = {}", addr);
    }

    fn enable_address(&self) {
        USBC_REGS.udcon.modify(DeviceControl::ADDEN::SET);

        debug!("Enable Address = {}", USBC_REGS.udcon.read(DeviceControl::UADD));
    }
}

impl<'a> Usbc<'a> {
    const fn new() -> Self {
        Usbc {
            client: None,
            state: MapCell::new(State::Reset),
            descriptors: [
                new_endpoint(),
                new_endpoint(),
                new_endpoint(),
                new_endpoint(),
                new_endpoint(),
                new_endpoint(),
                new_endpoint(),
                new_endpoint(),
            ],
        }
    }

    /// Attach to the USB bus after enabling USB clock
    fn _attach(&self) {
        self.state.map(|state| {
            match *state {
                State::Reset => {
                    client_err!("Not enabled");
                }
                State::Active(_) => {
                    client_err!("Already attached");
                }
                State::Idle(mode) => {
                    if pm::get_system_frequency() == 48000000 {
                        // XX: not clear that this always results in a usable USB clock
                        scif::generic_clock_enable(
                            scif::GenericClock::GCLK7,
                            scif::ClockSource::CLK_HSB,
                        );

                        while !USBC_REGS.usbsta.is_set(Status::CLKUSABLE) {}

                        USBC_REGS.udcon.modify(DeviceControl::DETACH::CLR);
                        debug!("Attached.");

                        *state = State::Active(mode);
                    } else {
                        debug!("The system clock does not support USB");
                    }
                }
            }
        });
    }

    /// Detach from the USB bus.  Also disable USB clock to save energy.
    fn _detach(&self) {
        self.state.map(|state| match *state {
            State::Reset => {
                client_err!("Not enabled");
            }
            State::Idle(_) => {
                client_err!("Not attached");
            }
            State::Active(mode) => {
                USBC_REGS.udcon.modify(DeviceControl::DETACH::SET);

                scif::generic_clock_disable(scif::GenericClock::GCLK7);

                *state = State::Idle(mode);
            }
        });
    }

    /// Enable the controller's clocks and interrupt and transition to Idle state
    /// (No effect if current state is not Reset)
    pub fn _enable(&self, mode: Mode) {
        self.state.map(|state| {
            match *state {
                State::Reset => {
                    // Are the USBC clocks enabled at reset?
                    //   10.7.4 says no, but 17.5.3 says yes
                    // Also, "Being in Idle state does not require the USB clocks to
                    //   be activated" (17.6.2)
                    enable_clock(Clock::HSB(HSBClock::USBC));
                    enable_clock(Clock::PBB(PBBClock::USBC));

                    // If we got to this state via disable() instead of chip reset,
                    // the values USBCON.FRZCLK, USBCON.UIMOD, UDCON.LS have *not* been
                    // reset to their default values.

                    if let Mode::Device { speed, .. } = mode {
                        USBC_REGS.udcon.modify(
                            match speed {
                                Speed::Full => DeviceConfig::LS::FullSpeed,
                                Speed::Low => DeviceConfig::LS::LowSpeed,
                            }
                        );
                    }

                    USBC_REGS.usbcon.modify(Control::UIMOD::DeviceMode);
                    USBC_REGS.usbcon.modify(Control::FRZCLK::CLR);
                    USBC_REGS.usbcon.modify(Control::USBE::SET);

                    USBC_REGS.udesc.set(&self.descriptors as *const _ as u32);

                    // Clear pending device global interrupts
                    USBC_REGS.udintclr.write(DeviceInterrupt::SUSP::SET +
                                             DeviceInterrupt::SOF::SET +
                                             DeviceInterrupt::EORST::SET +
                                             DeviceInterrupt::EORSM::SET +
                                             DeviceInterrupt::UPRSM::SET);

                    // Enable device global interrupts
                    USBC_REGS.udinteset.write(// DeviceInterrupt::SUSP::SET +
                                              // DeviceInterrupt::SOF::SET +
                                              DeviceInterrupt::EORST::SET +
                                              DeviceInterrupt::EORSM::SET +
                                              DeviceInterrupt::UPRSM::SET);

                    debug!("Enabled.");
                    *state = State::Idle(mode);
                }
                _ => client_err!("Already enabled"),
            }
        });
    }

    fn _active(&self) -> bool {
        self.state.map_or(false, |state| match *state {
            State::Active(_) => true,
            _ => false,
        })
    }

    /// Disable the controller, its interrupt, and its clocks
    fn _disable(&self) {
        if self._active() {
            self._detach();
        }

        self.state.map(|state| {
            if *state != State::Reset {
                USBC_REGS.usbcon.modify(Control::USBE::CLR);

                disable_clock(Clock::PBB(PBBClock::USBC));
                disable_clock(Clock::HSB(HSBClock::USBC));

                *state = State::Reset;
            }
        });
    }

    /// Provide a buffer for transfers in and out of the given endpoint
    pub fn endpoint_bank_set_buffer(
        &self,
        endpoint: EndpointIndex,
        bank: BankIndex,
        buf: &[VolatileCell<u8>],
    ) {
        let e: usize = From::from(endpoint);
        let b: usize = From::from(bank);
        let p = buf.as_ptr() as *mut u8;

        debug!("Set Endpoint{}/Bank{} addr={:8?}", e, b, p);
        self.descriptors[e][b].set_addr(p);
        self.descriptors[e][b].set_packet_size(PacketSize::default());
    }

    /// Configure and enable an endpoint
    /// (XX: include addr and packetsize?)
    pub fn endpoint_enable(&self, endpoint: u32, cfg: EndpointConfig) {
        self.state.map(|state| {
            // Record config in case of later reset
            match *state {
                State::Reset => {
                    client_err!("Not enabled");
                }
                State::Idle(Mode::Device { ref mut config, .. }) => {
                    *config = Some(cfg);
                }
                State::Active(Mode::Device { ref mut config, .. }) => {
                    *config = Some(cfg);
                }
                _ => {
                    client_err!("Not in Device mode");
                }
            }
        });

        // Enable the endpoint (meaning the controller will respond to requests)
        val eps = USBC_REGS.uerst.get();
        eps |= (1 << endpoint);
        USBC_REGS.uerst.set(eps);

        self.endpoint_configure(endpoint as usize, cfg);

        // Set EPnINTE, enabling interrupts for this endpoint
        let epints = USBC_REGS.udinteset.get();
        epints |= (1 << 12 + endpoint);
        USBC_REGS.udinteset.set(epints);

        debug!("Enabled endpoint {}", endpoint);
    }

    fn endpoint_configure(&self, endpoint: usize, cfg: EndpointConfig) {
        // Configure the endpoint
        USBC_REGS.uecfg[endpoint].set(From::from(cfg));

        // Specify which endpoint interrupts we want, among:
        //      TXIN | RXOUT | RXSTP | NAKOUT | NAKIN |
        //      ERRORF | STALLED | CRCERR | RAMACERR
        endpoint_enable_only_interrupts(endpoint, RXSTP | RAMACERR);

        // XX: Set endpoint state to Init
    }

    /// Set a client to receive data from the USBC
    pub fn set_client(&mut self, client: &'a hil::usb::Client) {
        self.client = Some(client);
    }

    /// Handle an interrupt from the USBC
    pub fn handle_interrupt(&mut self) {
        // TODO: Use a cell type with get_mut() so we don't have to copy the state value around
        let mut state = self.state.take().unwrap_or(State::Reset);

        match state {
            State::Reset => panic!("Not reached"),
            State::Idle(_) => panic!("Not reached"),
            State::Active(ref mut mode) => match *mode {
                Mode::Device {
                    speed,
                    ref config,
                    ref mut state,
                } => self.handle_device_interrupt(speed, config, state),
                Mode::Host => panic!("Unimplemented"),
            },
        }

        self.state.replace(state);
    }

    fn handle_device_interrupt(
        &mut self,
        speed: Speed,
        config: &Option<EndpointConfig>,
        dstate: &mut DeviceState,
    ) {
        let udint = USBC_REGS.udint.cache();

        // debug!("--> UDINT={:?} {:?}", UdintFlags(udint), *dstate);

        if udint.is_set(DeviceInterrupt::EORST) {
            // Bus reset

            // Reconfigure what has been reset in the USBC
            USBC_REGS.udcon.modify(
                match speed {
                    Speed::Full => DeviceConfig::LS::FullSpeed,
                    Speed::Low => DeviceConfig::LS::LowSpeed,
                }
            );
            if let Some(ref config) = *config {
                self.endpoint_configure(0, *config);
            }

            // Re-initialize our record of the controller state
            *dstate = DeviceState::Init;

            // Alert the client
            self.client.map(|client| {
                client.bus_reset();
            });
            debug!("USB Bus Reset");
            // debug_regs();

            // Acknowledge the interrupt
            USBC_REGS.udintclr.write(DeviceInterrupt::EORST::SET);
        }

        if udint.is_set(DeviceInterrupt::SUSP) {
            // The transceiver has been suspended due to the bus being idle for 3ms.
            // This condition is over when WAKEUP is set.

            // "To further reduce power consumption it is recommended to freeze the USB
            // clock by writing a one to the Freeze USB Clock (FRZCLK) bit in USBCON when
            // the USB bus is in suspend mode.
            //
            // To recover from the suspend mode, the user shall wait for the Wakeup
            // (WAKEUP) interrupt bit, which is set when a non-idle event is detected, and
            // then write a zero to FRZCLK.
            //
            // As the WAKEUP interrupt bit in UDINT is set when a non-idle event is
            // detected, it can occur regardless of whether the controller is in the
            // suspend mode or not."

            // Subscribe to WAKEUP
            USBC_REGS.udinteset.write(DeviceInterrupt::WAKEUP::SET);

            // Acknowledge the "suspend" event
            USBC_REGS.udintclr.write(DeviceInterrupt::SUSP::SET);
        }

        if udint.is_set(DeviceInterrupt::WAKEUP) {
            // If we were suspended: Unfreeze the clock (and unsleep the MCU)

            // Unsubscribe from WAKEUP
            USBC_REGS.udinteclr.write(DeviceInterrupt::WAKEUP::SET);

            // Acknowledge the interrupt
            USBC_REGS.udintclr.write(DeviceInterrupt::WAKEUP::SET);

            // Continue processing, as WAKEUP is usually set
        }

        if udint.is_set(DeviceInterrupt::SOF) {
            // Acknowledge Start of frame
            USBC_REGS.udintclr.write(DeviceInterrupt::SOF::SET);
        }

        if udint.is_set(DeviceInterrupt::EORSM) {
            // Controller received End of Resume
            debug!("UDINT EORSM");
        }

        if udint.is_set(DeviceInterrupt::UPRSM) {
            // Controller sent Upstream Resume
            debug!("UDINT UPRSM");
        }

        // Process per-endpoint interrupt flags
        for endpoint in 0..1 {
            if udint.get() & (1 << (12 + endpoint)) == 0 {
                // No interrupts for this endpoint
                continue;
            }

            // Set to true to process more flags without waiting for another interrupt
            // (Using this with debugging messages tends to fill up the console buffer too fast.)
            // (Ignoring `again` should not cause incorrect behavior.)
            //
            // let mut again = true;
            /* while again */ {
                // again = false;

                let status = USBC_REGS.uesta[endpoint].cache();
                // debug!("UESTA{}={:?}", endpoint, UestaFlags(status));

                if status.is_set(EndpointStatus::STALLED) {
                    debug!("D({}) STALLED/CRCERR", endpoint);

                    // Acknowledge
                    USBC_REGS.uestaclr[endpoint].write(EndpointStatus::STALLED::SET);
                }

                if status.is_set(EndpointStatus::RAMACERR) {
                    debug!("D({}) RAMACERR", endpoint);

                    // Acknowledge
                    USBC_REGS.uestaclr[endpoint].write(EndpointStatus::RAMACERR::SET);
                }

                match *dstate {
                    DeviceState::Init => {
                        if status.is_set(EndpointStatus::RXSTP) {
                            // We received a SETUP transaction

                            // debug!("D({}) RXSTP", endpoint);
                            // self.debug_show_d0();

                            let packet_bytes =
                                self.descriptors[0][0].packet_size.get().byte_count();
                            let result = if packet_bytes == 8 {
                                self.client.map(|c| c.ctrl_setup())
                            } else {
                                Some(CtrlSetupResult::ErrBadLength)
                            };

                            match result {
                                Some(CtrlSetupResult::Ok) => {
                                    if status.read(EndpointStatus::CTRLDIR) == EndpointStatus::CTRLDIR::In {
                                        // The following Data stage will be IN

                                        *dstate = DeviceState::CtrlReadIn;

                                        // Wait until bank is clear to send
                                        // Also, wait for NAKOUT to signal end of IN stage
                                        // (The datasheet incorrectly says NAKIN)
                                        USBC_REG.uestaclr[endpoint].write(EndpointStatus::NAKOUT::SET);
                                        endpoint_enable_only_interrupts(
                                            endpoint,
                                            RAMACERR | TXIN | NAKOUT,
                                        );
                                    } else {
                                        // The following Data stage will be OUT

                                        *dstate = DeviceState::CtrlWriteOut;

                                        // Wait for OUT packets
                                        // Also, wait for NAKIN to signal end of OUT stage
                                        USBC_REG.uestaclr[endpoint].write(EndpointStatus::RXOUT::SET +
                                                                          EndpointStatus::NAKIN::SET);

                                        endpoint_enable_only_interrupts(
                                            endpoint,
                                            RAMACERR | RXOUT | NAKIN,
                                        );
                                    }
                                }
                                failure => {
                                    // Respond with STALL to any following transactions
                                    // in this request
                                    USBC_REG.ueconset[endpoint].write(EndpointControl::STALLRQ::SET);

                                    match failure {
                                        None => debug!("D({}) No client to handle Setup", endpoint),
                                        Some(err) => {
                                            debug!("D({}) Client err on Setup: {:?}", endpoint, err)
                                        }
                                    }

                                    endpoint_enable_only_interrupts(endpoint, RXSTP | RAMACERR);

                                    // Remain in DeviceState::Init for next SETUP
                                }
                            }

                            // Acknowledge SETUP interrupt
                            USBC_REG.uestaclr[endpoint].write(EndpointStatus::RXSTP::SET);
                        }
                    }
                    DeviceState::CtrlReadIn => {
                        if status.is_set(EndpointStatus::NAKOUT) {
                            // The host has completed the IN stage by sending an OUT token

                            endpoint_disable_interrupts(endpoint, TXIN | NAKOUT);

                            // debug!("D({}) NAKOUT");
                            self.client.map(|c| c.ctrl_status());

                            *dstate = DeviceState::CtrlReadStatus;

                            // Await end of Status stage
                            endpoint_enable_interrupts(endpoint, RXOUT);

                            // Acknowledge
                            UESTAnCLR[endpoint].write(NAKOUT);

                        // Run handler again in case the RXOUT has already arrived
                        // again = true;
                        } else if status & TXIN != 0 {
                            // The data bank is ready to receive another IN payload
                            // debug!("D({}) TXIN", endpoint);

                            let result = self.client.map(|c| {
                                // Allow client to write a packet payload to buffer
                                c.ctrl_in()
                            });
                            match result {
                                Some(CtrlInResult::Packet(packet_bytes, transfer_complete)) => {
                                    self.descriptors[0][0].packet_size.set(if packet_bytes == 8
                                        && transfer_complete
                                    {
                                        // Send a complete final packet, and request
                                        // that the controller also send a zero-length
                                        // packet to signal the end of transfer
                                        PacketSize::single_with_zlp(8)
                                    } else {
                                        // Send either a complete but not-final
                                        // packet, or a short and final packet (which
                                        // itself signals end of transfer)
                                        PacketSize::single(packet_bytes as u32)
                                    });

                                    // debug!("D({}) Send CTRL IN packet ({} bytes)",
                                    //        endpoint,
                                    //        packet_bytes);
                                    // self.debug_show_d0();

                                    if transfer_complete {
                                        // IN data completely sent.  Unsubscribe from TXIN.
                                        // (Continue awaiting NAKOUT to indicate end of Data stage)
                                        endpoint_disable_interrupts(endpoint, TXIN);
                                    } else {
                                        // Continue waiting for next TXIN
                                    }

                                    // Signal to the controller that the IN payload is
                                    // ready to send
                                    UESTAnCLR[endpoint].write(TXIN);
                                }
                                Some(CtrlInResult::Delay) => {
                                    endpoint_disable_interrupts(endpoint, TXIN);
                                    debug!("*** Client NAK");
                                    // XXX set busy bits?
                                    *dstate = DeviceState::CtrlInDelay;
                                }
                                _ => {
                                    // Respond with STALL to any following IN/OUT transactions
                                    UECONnSET[endpoint].write(STALLRQ);

                                    debug!("D({}) Client IN err => STALL", endpoint);

                                    *dstate = DeviceState::Init;

                                    // Wait for next SETUP
                                    endpoint_enable_interrupts(endpoint, RXSTP);
                                }
                            }
                        }
                    }
                    DeviceState::CtrlReadStatus => {
                        if status & RXOUT != 0 {
                            // Host has completed Status stage by sending an OUT packet

                            endpoint_disable_interrupts(endpoint, RXOUT);

                            // debug!("D({}) RXOUT: End of Control Read transaction", endpoint);
                            self.client.map(|c| c.ctrl_status_complete());

                            *dstate = DeviceState::Init;

                            // Wait for next SETUP
                            endpoint_enable_interrupts(endpoint, RXSTP);

                            // Acknowledge
                            UESTAnCLR[endpoint].write(RXOUT);
                        }
                    }
                    DeviceState::CtrlWriteOut => {
                        if status & RXOUT != 0 {
                            // Received data

                            // debug!("D({}) RXOUT: Received Control Write data", endpoint);
                            // self.debug_show_d0();
                            let result = self.client.map(|c| {
                                c.ctrl_out(self.descriptors[0][0].packet_size.get().byte_count())
                            });
                            match result {
                                Some(CtrlOutResult::Ok) => {
                                    // Acknowledge
                                    UESTAnCLR[endpoint].write(RXOUT);
                                }
                                Some(CtrlOutResult::Delay) => {
                                    // Don't acknowledge; hardware will have to send NAK

                                    // Unsubscribe from RXOUT until client says it is ready
                                    // (But there is not yet any interface for that)
                                    endpoint_disable_interrupts(endpoint, RXOUT);
                                }
                                _ => {
                                    // Respond with STALL to any following transactions
                                    // in this request
                                    UECONnSET[endpoint].write(STALLRQ);

                                    debug!("D({}) Client OUT err => STALL", endpoint);

                                    *dstate = DeviceState::Init;

                                    // Wait for next SETUP
                                    endpoint_enable_interrupts(endpoint, RXSTP);
                                }
                            }

                            // Continue awaiting RXOUT and NAKIN
                        }
                        if status & NAKIN != 0 {
                            // The host has completed the Data stage by sending an IN token
                            // debug!("D({}) NAKIN: Control Write -> Status stage", endpoint);

                            endpoint_disable_interrupts(endpoint, RXOUT | NAKIN);

                            *dstate = DeviceState::CtrlWriteStatus;

                            // Wait for bank to be free so we can write ZLP to acknowledge transfer
                            endpoint_enable_interrupts(endpoint, TXIN);

                            // Acknowledge
                            UESTAnCLR[endpoint].write(NAKIN);

                            // Can probably send the ZLP immediately
                            // again = true;
                        }
                    }
                    DeviceState::CtrlWriteStatus => {
                        if status & TXIN != 0 {
                            // debug!("D({}) TXIN for Control Write Status (will send ZLP)",
                            //        endpoint);

                            self.client.map(|c| c.ctrl_status());

                            // Send zero-length packet to acknowledge transaction
                            self.descriptors[0][0]
                                .packet_size
                                .set(PacketSize::single(0));

                            *dstate = DeviceState::CtrlWriteStatusWait;

                            // Signal to the controller that the IN payload is ready to send
                            UESTAnCLR[endpoint].write(TXIN);

                            // Wait for TXIN again to confirm that IN payload has been sent
                        }
                    }
                    DeviceState::CtrlWriteStatusWait => {
                        if status & TXIN != 0 {
                            // debug!("D({}) TXIN: Control Write Status Complete", endpoint);

                            endpoint_disable_interrupts(endpoint, TXIN);

                            *dstate = DeviceState::Init;

                            // Wait for next SETUP
                            endpoint_enable_interrupts(endpoint, RXSTP);

                            // for SetAddress, client must enable address after STATUS stage
                            self.client.map(|c| c.ctrl_status_complete());
                        }
                    }
                    DeviceState::CtrlInDelay => { /* XX: Spin fruitlessly */ }
                } // match dstate

                // again = false; // XX
            } // while again
        } // for endpoint
    } // handle_device_interrupt

    #[allow(dead_code)]
    fn debug_show_d0(&self) {
        for bi in 0..1 {
            let b = &self.descriptors[0][bi];
            let addr = b.addr.get();
            let buf = if addr.is_null() {
                None
            } else {
                unsafe {
                    Some(slice::from_raw_parts(
                        addr,
                        b.packet_size.get().byte_count() as usize,
                    ))
                }
            };

            debug!(
                "B_0_{} \
                 \n     {:?}\
                 \n     {:?}\
                 \n     {:?}",
                bi, // (&b.addr as *const _), b.addr.get(),
                b.packet_size.get(),
                b.ctrl_status.get(),
                buf.map(HexBuf)
            );
        }
    }

    pub fn mode(&self) -> Option<Mode> {
        self.state.map_or(None, |state| match *state {
            State::Idle(mode) => Some(mode),
            State::Active(mode) => Some(mode),
            _ => None,
        })
    }

    pub fn speed(&self) -> Option<Speed> {
        match self.mode() {
            Some(mode) => {
                match mode {
                    Mode::Device { speed, .. } => Some(speed),
                    Mode::Host => {
                        None // XX USBSTA.SPEED
                    }
                }
            }
            _ => None,
        }
    }

    // Remote wakeup (Device -> Host, after receiving DEVICE_REMOTE_WAKEUP)
}

#[inline]
fn endpoint_disable_interrupts(endpoint: usize, mask: u32) {
    UECONnCLR[endpoint].write(mask);
}

#[inline]
fn endpoint_enable_interrupts(endpoint: usize, mask: u32) {
    UECONnSET[endpoint].write(mask);
}

#[inline]
fn endpoint_enable_only_interrupts(endpoint: usize, mask: u32) {
    endpoint_disable_interrupts(endpoint, !0);
    endpoint_enable_interrupts(endpoint, mask);
}

#[allow(dead_code)]
fn debug_regs() {
    debug!(
        "    registers:\
         \n    USBFSM={:08x}\
         \n    USBCON={:08x}\
         \n    USBSTA={:08x}\
         \n     UDESC={:08x}\
         \n     UDCON={:08x}\
         \n    UDINTE={:08x}\
         \n     UDINT={:08x}\
         \n     UERST={:08x}\
         \n    UECFG0={:08x}\
         \n    UECON0={:08x}",
        USBFSM.read(),
        USBCON.read(),
        USBSTA.read(),
        UDESC.read(),
        UDCON.read(),
        UDINTE.read(),
        UDINT.read(),
        UERST.read(),
        UECFG0.read(),
        UECON0.read()
    );
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
            write!(f, "STALLED");
        }
        if w & CRCERR != 0 {
            write!(f, "/CRCERR ");
        }
        if w & RAMACERR != 0 {
            write!(f, "RAMACERR ");
        }
        write!(f, "NBUSYBK={} ", (w >> 12) & 0x3);
        write!(f, "CURBK={} ", (w >> 14) & 0x3);
        write!(f, "CTRLDIR={}", if w & CTRLDIR != 0 { "IN" } else { "OUT" });
        write!(f, "}}")
    }
}

/// Static state to manage the USBC
pub static mut USBC: Usbc<'static> = Usbc::new();
