//! SAM4L USB controller

pub mod debug;

#[allow(unused_imports)]
use self::debug::{UdintFlags, UestaFlags};
use core::cell::Cell;
use core::slice;
use core::ptr;
use kernel::StaticRef;
use kernel::common::VolatileCell;
use kernel::common::regs::{RegisterValue, FieldValue, ReadOnly, ReadWrite, WriteOnly};
use kernel::hil;
use kernel::hil::usb::*;
use pm;
use pm::{disable_clock, enable_clock, Clock, HSBClock, PBBClock};
use scif;

// The following macros provide some diagnostics and panics(!)
// while this module is experimental and should eventually be removed or
// replaced with better error handling.

macro_rules! client_warn {
    [ $( $arg:expr ),+ ] => {
        debug!($( $arg ),+);
    };
}

macro_rules! client_err {
    [ $( $arg:expr ),+ ] => {
        panic!($( $arg ),+);
    };
}

macro_rules! debug1 {
    [ $( $arg:expr ),+ ] => {
        {} // debug!($( $arg ),+)
    };
}

macro_rules! internal_err {
    [ $( $arg:expr ),+ ] => {
        panic!($( $arg ),+);
    };
}

#[repr(C)]
struct UsbcRegisters {
    udcon: ReadWrite<u32, DeviceControl::Register>,
    udint: ReadOnly<u32, DeviceInterrupt::Register>,
    udintclr: WriteOnly<u32, DeviceInterrupt::Register>,
    udintset: WriteOnly<u32, DeviceInterrupt::Register>,
    udinte: ReadOnly<u32, DeviceInterrupt::Register>,
    udinteclr: WriteOnly<u32, DeviceInterrupt::Register>,
    udinteset: WriteOnly<u32, DeviceInterrupt::Register>,
    uerst: ReadWrite<u32>,
    udfnum: ReadOnly<u32>,
    _reserved0: [u8; 0xdc], // 220 bytes
    // Note that the SAM4L supports only 8 endpoints, but the registers
    // are laid out such that there is room for 12.
    // 0x100
    uecfg: [ReadWrite<u32, EndpointConfig::Register>; 12],
    uesta: [ReadOnly<u32, EndpointStatus::Register>; 12],
    uestaclr: [WriteOnly<u32, EndpointStatus::Register>; 12],
    uestaset: [WriteOnly<u32, EndpointStatus::Register>; 12],
    uecon: [ReadOnly<u32, EndpointControl::Register>; 12],
    ueconset: [WriteOnly<u32, EndpointControl::Register>; 12],
    ueconclr: [WriteOnly<u32, EndpointControl::Register>; 12],
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
    usbcon: ReadWrite<u32, Control::Register>,
    usbsta: ReadOnly<u32, Status::Register>,
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
        UIMOD OFFSET(25) NUMBITS(1) [
            HostMode = 0,
            DeviceMode = 1
        ],
        USBE OFFSET(15) NUMBITS(1) [],
        FRZCLK OFFSET(14) NUMBITS(1) []
    ],
    Status [
        SUSPEND OFFSET(16) NUMBITS(1) [],
        CLKUSABLE OFFSET(14) NUMBITS(1) [],
        SPEED OFFSET(12) NUMBITS(2) [
            SpeedFull = 0b00,
            SpeedLow = 0b10
        ],
        VBUSRQ OFFSET(9) NUMBITS(1) []
    ],
    DeviceControl [
        GNAK OFFSET(17) NUMBITS(1) [],
        LS OFFSET(12) NUMBITS(1) [
            FullSpeed = 0,
            LowSpeed = 1
        ],
        RMWKUP OFFSET(9) NUMBITS(1) [],
        DETACH OFFSET(8) NUMBITS(1) [],
        ADDEN OFFSET(7) NUMBITS(1) [],
        UADD OFFSET(0) NUMBITS(7) []
    ],
    DeviceInterrupt [
        EPINT OFFSET(12) NUMBITS(8),
        UPRSM OFFSET(6) NUMBITS(1),
        EORSM OFFSET(5) NUMBITS(1),
        WAKEUP OFFSET(4) NUMBITS(1),
        EORST OFFSET(3) NUMBITS(1),
        SOF OFFSET(2) NUMBITS(1),
        SUSP OFFSET(0) NUMBITS(1)
    ],
    EndpointConfig [
        REPNB OFFSET(16) NUMBITS(4) [
            NotRedirected = 0
        ],
        EPTYPE OFFSET(11) NUMBITS(2) [
            Control = 0,
            Isochronous = 1,
            Bulk = 2,
            Interrupt = 3
        ],
        EPDIR OFFSET(8) NUMBITS(1) [
            Out = 0,
            In = 1
        ],
        EPSIZE OFFSET(4) NUMBITS(3) [
            Bytes8 = 0,
            Bytes16 = 1,
            Bytes32 = 2,
            Bytes64 = 3,
            Bytes128 = 4,
            Bytes256 = 5,
            Bytes512 = 6,
            Bytes1024 = 7
        ],
        EPBK OFFSET(2) NUMBITS(1) [
            Single = 0,
            Double = 1
        ]
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
        NBUSYBK OFFSET(12) NUMBITS(2) [],
        RAMACER OFFSET(11) NUMBITS(1) [],
        DTSEQ OFFSET(8) NUMBITS(2) [
            Data0 = 0,
            Data1 = 1
        ],
        STALLED OFFSET(6) NUMBITS(1) [],
        CRCERR OFFSET(6) NUMBITS(1) [],
        NAKIN OFFSET(4) NUMBITS(1) [],
        NAKOUT OFFSET(3) NUMBITS(1) [],
        ERRORF OFFSET(2) NUMBITS(1) [],
        RXSTP OFFSET(2) NUMBITS(1) [],
        RXOUT OFFSET(1) NUMBITS(1) [],
        TXIN OFFSET(0) NUMBITS(1) []
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

const USBC_BASE: StaticRef<UsbcRegisters> =
    unsafe { StaticRef::new(0x400A5000 as *const UsbcRegisters) };

#[inline]
fn usbc_regs() -> &'static UsbcRegisters {
    &*USBC_BASE
}

// Datastructures for tracking USB controller state

// This ensures the `descriptors` field is laid out first
#[repr(C)]
// This provides the required alignment for the `descriptors` field
#[repr(align(8))]
pub struct Usbc<'a> {
    descriptors: [Endpoint; 8],
    state: Cell<State>,
    client: Option<&'a hil::usb::Client>,
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    // Controller disabled
    Reset,

    // Controller enabled, detached from bus
    // (We may go to this state when the Host
    // controller suspends the bus.)
    Idle(Mode),

    // Controller enabled, attached to bus
    Active(Mode),
}

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Host,
    Device {
        speed: Speed,
        config: DeviceConfig,
        state: DeviceState,
    },
}

pub const N_ENDPOINTS: usize = 8;

type EndpointConfigValue = RegisterValue<u32, EndpointConfig::Register>;
type EndpointStatusValue = RegisterValue<u32, EndpointStatus::Register>;

#[derive(Copy, Clone, Debug, Default)]
pub struct DeviceConfig {
    pub endpoint_configs: [Option<EndpointConfigValue>; N_ENDPOINTS],
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DeviceState {
    pub endpoint_states: [EndpointState; N_ENDPOINTS],
}

#[derive(Copy, Clone, Debug)]
pub enum EndpointState {
    Disabled,
    Ctrl(CtrlState),
    Bulk(BulkState),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CtrlState {
    Init,
    ReadIn,
    ReadStatus,
    WriteOut,
    WriteStatus,
    WriteStatusWait,
    InDelay,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkState {
    Init,
    InDelay,
}

impl Default for EndpointState {
    fn default() -> Self {
        EndpointState::Disabled
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Speed {
    Full,
    Low,
}

pub enum BankIndex {
    Bank0,
    Bank1,
}

impl From<BankIndex> for usize {
    fn from(bi: BankIndex) -> usize {
        match bi {
            BankIndex::Bank0 => 0,
            BankIndex::Bank1 => 1,
        }
    }
}

pub struct EndpointIndex(u8);

impl EndpointIndex {
    pub fn new(index: usize) -> EndpointIndex {
        EndpointIndex(index as u8 & 0xf)
    }

    pub fn to_u32(self) -> u32 {
        self.0 as u32
    }
}

impl From<EndpointIndex> for usize {
    fn from(ei: EndpointIndex) -> usize {
        ei.0 as usize
    }
}

pub type Endpoint = [Bank; 2];

pub const fn new_endpoint() -> Endpoint {
    [Bank::new(), Bank::new()]
}

#[repr(C)]
pub struct Bank {
    addr: VolatileCell<*mut u8>,

    // The following fields are not actually registers
    // (they may be placed anywhere in memory),
    // but the register interface provides the volatile
    // read/writes and bitfields that we need.
    pub packet_size: ReadWrite<u32, PacketSize::Register>,
    pub control_status: ReadWrite<u32, ControlStatus::Register>,

    _reserved: u32,
}

impl Bank {
    pub const fn new() -> Bank {
        Bank {
            addr: VolatileCell::new(ptr::null_mut()),
            packet_size: ReadWrite::new(0),
            control_status: ReadWrite::new(0),
            _reserved: 0,
        }
    }

    pub fn set_addr(&self, addr: *mut u8) {
        self.addr.set(addr);
    }
}

register_bitfields![u32,
    PacketSize [
        AUTO_ZLP OFFSET(31) NUMBITS(1) [
            No = 0,
            Yes = 1
        ],
        MULTI_PACKET_SIZE OFFSET(16) NUMBITS(15) [],
        BYTE_COUNT OFFSET(0) NUMBITS(15) []
    ],
    ControlStatus [
        UNDERF 18,
        OVERF 17,
        CRCERR 16,
        STALLRQ_NEXT 0
    ]
];

impl<'a> Usbc<'a> {
    const fn new() -> Self {
        Usbc {
            client: None,
            state: Cell::new(State::Reset),
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

    fn map_state<F, R>(&self, closure: F) -> R
    where
        F: FnOnce(&mut State) -> R,
    {
        let mut state = self.state.get();
        let result = closure(&mut state);
        self.state.set(state);
        result
    }

    fn get_state(&self) -> State {
        self.state.get()
    }

    fn set_state(&self, state: State) {
        self.state.set(state);
    }

    /// Provide a buffer for transfers in and out of the given endpoint
    /// (The controller need not be enabled before calling this method.)
    fn _endpoint_bank_set_buffer(
        &self,
        endpoint: EndpointIndex,
        bank: BankIndex,
        buf: &[VolatileCell<u8>],
    ) {
        let e: usize = From::from(endpoint);
        let b: usize = From::from(bank);
        let p = buf.as_ptr() as *mut u8;

        debug1!("Set Endpoint{}/Bank{} addr={:8?}", e, b, p);
        self.descriptors[e][b].set_addr(p);
        self.descriptors[e][b].packet_size.write(PacketSize::BYTE_COUNT.val(0) +
                                                 PacketSize::MULTI_PACKET_SIZE.val(0) +
                                                 PacketSize::AUTO_ZLP::No);
    }

    /// Enable the controller's clocks and interrupt and transition to Idle state
    fn _enable(&self, mode: Mode) {
        match self.get_state() {
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
                    usbc_regs().udcon.modify(match speed {
                        Speed::Full => DeviceControl::LS::FullSpeed,
                        Speed::Low => DeviceControl::LS::LowSpeed,
                    });
                }

                // Enable in device mode
                usbc_regs().usbcon.modify(Control::UIMOD::DeviceMode);
                usbc_regs().usbcon.modify(Control::FRZCLK::CLEAR);
                usbc_regs().usbcon.modify(Control::USBE::SET);

                // Set the pointer to the endpoint descriptors
                usbc_regs().udesc.set(&self.descriptors as *const _ as u32);

                // Clear pending device global interrupts
                usbc_regs().udintclr.write(
                    DeviceInterrupt::SUSP::SET + DeviceInterrupt::SOF::SET
                        + DeviceInterrupt::EORST::SET
                        + DeviceInterrupt::EORSM::SET
                        + DeviceInterrupt::UPRSM::SET,
                );

                // Enable device global interrupts
                // Note: SOF has been omitted as it is not presently used,
                // Note: SUSP has been omitted as SUSPEND/WAKEUP is not yet
                //   implemented.
                // Note: SOF and SUSP may nevertheless be enabled here
                //   without harm, but it makes debugging easier to omit them.
                usbc_regs().udinteset.write(
                    DeviceInterrupt::EORST::SET + DeviceInterrupt::EORSM::SET
                        + DeviceInterrupt::UPRSM::SET,
                );

                debug1!("Enabled");

                self.set_state(State::Idle(mode));
            }
            _ => internal_err!("Already enabled"),
        }
    }

    /// Disable the controller, its interrupt, and its clocks
    fn _disable(&self) {
        // Detach if necessary
        if let State::Active(_) = self.get_state() {
            self._detach();
        }

        // Disable USBC and its clocks
        match self.get_state() {
            State::Idle(..) => {
                usbc_regs().usbcon.modify(Control::USBE::CLEAR);

                disable_clock(Clock::PBB(PBBClock::USBC));
                disable_clock(Clock::HSB(HSBClock::USBC));

                self.set_state(State::Reset);
            }
            _ => internal_err!("Disable called from wrong state"),
        }
    }

    /// Attach to the USB bus after enabling USB bus clock
    fn _attach(&self) {
        match self.get_state() {
            State::Idle(mode) => {
                if pm::get_system_frequency() != 48000000 {
                    internal_err!("The system clock does not support USB");
                }

                // XX: not clear that this always results in a usable USB clock
                scif::generic_clock_enable(scif::GenericClock::GCLK7, scif::ClockSource::CLK_HSB);

                while !usbc_regs().usbsta.is_set(Status::CLKUSABLE) {}

                usbc_regs().udcon.modify(DeviceControl::DETACH::CLEAR);

                debug1!("Attached");

                self.set_state(State::Active(mode));
            }
            _ => internal_err!("Attach called in wrong state"),
        }
    }

    /// Detach from the USB bus.  Also disable USB bus clock to save energy.
    fn _detach(&self) {
        match self.get_state() {
            State::Active(mode) => {
                usbc_regs().udcon.modify(DeviceControl::DETACH::SET);

                scif::generic_clock_disable(scif::GenericClock::GCLK7);

                self.set_state(State::Idle(mode));
            }
            _ => debug1!("Already detached"),
        }
    }

    /// Configure and enable an endpoint
    fn _endpoint_enable(&self, endpoint: usize, endpoint_config: EndpointConfigValue) {
        // Record config in case of later reset
        self.map_state(|state| match *state {
            State::Reset => {
                internal_err!("Not enabled");
            }
            State::Idle(Mode::Device { ref mut config, .. }) => {
                config.endpoint_configs[endpoint] = Some(endpoint_config);
            }
            State::Active(Mode::Device { ref mut config, .. }) => {
                config.endpoint_configs[endpoint] = Some(endpoint_config);
            }
            _ => internal_err!("Not in Device mode"),
        });

        self._endpoint_config(endpoint, endpoint_config);

        // Enable the endpoint (meaning the controller will respond to requests
        // to this endpoint)
        usbc_regs()
            .uerst
            .set(usbc_regs().uerst.get() | (1 << endpoint));

        self._endpoint_init(endpoint, endpoint_config);

        // Set EPnINTE, enabling interrupts for this endpoint
        usbc_regs().udinteset.set(1 << (12 + endpoint));

        debug1!("Enabled endpoint {}", endpoint);
    }

    fn _endpoint_config(&self, endpoint: usize, config: EndpointConfigValue) {
        // This must be performed after each bus reset

        // Configure the endpoint
        usbc_regs().uecfg[endpoint].set(From::from(config));

        debug1!("Configured endpoint {}", endpoint);
    }

    fn _endpoint_init(&self, endpoint: usize, config: EndpointConfigValue) {
        self.map_state(|state| match *state {
            State::Idle(Mode::Device { ref mut state, .. }) => {
                self._endpoint_init_device_state(state, endpoint, config);
            }
            State::Active(Mode::Device { ref mut state, .. }) => {
                self._endpoint_init_device_state(state, endpoint, config);
            }
            _ => internal_err!("Not reached"),
        });
    }

    fn _endpoint_init_device_state(
        &self,
        state: &mut DeviceState,
        endpoint: usize,
        config: EndpointConfigValue,
    ) {
        // This must be performed after each bus reset (see 17.6.2.2)

        endpoint_enable_interrupts(endpoint, EndpointControl::RAMACERE::SET + EndpointControl::STALLEDE::SET);

        if config.matches_all(EndpointConfig::EPTYPE::Control) {
            endpoint_enable_interrupts(endpoint, EndpointControl::TXINE::SET);
            state.endpoint_states[endpoint] = EndpointState::Ctrl(CtrlState::Init);
        }
        else if config.matches_all(EndpointConfig::EPTYPE::Bulk) {
            endpoint_enable_interrupts(endpoint, EndpointControl::TXINE::SET);
            state.endpoint_states[endpoint] = EndpointState::Bulk(BulkState::Init);
        }
        else {
            // Other endpoint types unimplemented
        }

        debug1!("Initialized endpoint {}", endpoint);
    }

    /// Set a client to receive data from the USBC
    pub fn set_client(&mut self, client: &'a hil::usb::Client) {
        self.client = Some(client);
    }

    /// Handle an interrupt from the USBC
    pub fn handle_interrupt(&self) {
        self.map_state(|state| match *state {
            State::Reset => internal_err!("Received interrupt in Reset"),
            State::Idle(_) => {
                // We might process WAKEUP here
                debug1!("Received interrupt in Idle");
            }
            State::Active(ref mut mode) => match *mode {
                Mode::Device {
                    speed,
                    ref config,
                    ref mut state,
                } => self.handle_device_interrupt(speed, config, state),
                Mode::Host => internal_err!("Host mode unimplemented"),
            },
        });
    }

    /// Handle an interrupt while in device mode
    fn handle_device_interrupt(
        &self,
        speed: Speed,
        device_config: &DeviceConfig,
        device_state: &mut DeviceState,
    ) {
        let udint = usbc_regs().udint.get_value();

        debug1!(
            "--> UDINT={:?} ep0:{:?}",
            UdintFlags(udint.get()),
            device_state.endpoint_states[0]
        );

        if udint.is_set(DeviceInterrupt::EORST) {
            // Bus reset
            debug1!("USB Bus Reset");

            // Reconfigure what has been reset in the USBC
            usbc_regs().udcon.modify(match speed {
                Speed::Full => DeviceControl::LS::FullSpeed,
                Speed::Low => DeviceControl::LS::LowSpeed,
            });

            // Reset our record of the device state
            *device_state = Default::default();

            // Reconfigure and initialize endpoints
            for i in 0..N_ENDPOINTS {
                if let Some(endpoint_config) = device_config.endpoint_configs[i] {
                    self._endpoint_config(i, endpoint_config);
                    self._endpoint_init_device_state(device_state, i, endpoint_config);
                }
            }

            // Alert the client
            self.client.map(|client| {
                client.bus_reset();
            });

            // Acknowledge the interrupt
            usbc_regs().udintclr.write(DeviceInterrupt::EORST::SET);

            // Wait for the next interrupt before doing anything else
            return;
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
            usbc_regs().udinteset.write(DeviceInterrupt::WAKEUP::SET);

            // Acknowledge the "suspend" event
            usbc_regs().udintclr.write(DeviceInterrupt::SUSP::SET);
        }

        if udint.is_set(DeviceInterrupt::WAKEUP) {
            // If we were suspended: Unfreeze the clock (and unsleep the MCU)

            // Unsubscribe from WAKEUP
            usbc_regs().udinteclr.write(DeviceInterrupt::WAKEUP::SET);

            // Acknowledge the interrupt
            usbc_regs().udintclr.write(DeviceInterrupt::WAKEUP::SET);

            // Continue processing, as WAKEUP is usually set
        }

        if udint.is_set(DeviceInterrupt::SOF) {
            // Acknowledge Start of frame
            usbc_regs().udintclr.write(DeviceInterrupt::SOF::SET);
        }

        if udint.is_set(DeviceInterrupt::EORSM) {
            // Controller received End of Resume
            debug1!("UDINT EORSM");
        }

        if udint.is_set(DeviceInterrupt::UPRSM) {
            // Controller sent Upstream Resume
            debug1!("UDINT UPRSM");
        }

        // Process per-endpoint interrupt flags
        for endpoint in 0..N_ENDPOINTS {
            if udint.get() & (1 << (12 + endpoint)) == 0 {
                // No interrupts for this endpoint
                continue;
            }

            self.handle_endpoint_interrupt(endpoint, &mut device_state.endpoint_states[endpoint]);
        }
    }

    fn handle_endpoint_interrupt(&self, endpoint: usize, endpoint_state: &mut EndpointState) {
        let status = usbc_regs().uesta[endpoint].get_value();
        debug1!("UESTA{}={:?}", endpoint, UestaFlags(status.get()));

        if status.is_set(EndpointStatus::STALLED) {
            debug1!("D({}) STALLED/CRCERR", endpoint);

            // Acknowledge
            usbc_regs().uestaclr[endpoint].write(EndpointStatus::STALLED::SET);
        }

        if status.is_set(EndpointStatus::RAMACER) {
            debug1!("D({}) RAMACER", endpoint);

            // Acknowledge
            usbc_regs().uestaclr[endpoint].write(EndpointStatus::RAMACER::SET);
        }

        match *endpoint_state {
            EndpointState::Ctrl(ref mut ctrl_state) =>
                self.handle_ctrl_endpoint_interrupt(endpoint, ctrl_state, status),
            EndpointState::Bulk(ref mut bulk_state) =>
                self.handle_bulk_endpoint_interrupt(endpoint, bulk_state, status),
            EndpointState::Disabled => {
                debug1!("Ignoring interrupt for disabled endpoint {}", endpoint);
                return;
            }
        }
    }

    fn handle_ctrl_endpoint_interrupt(
        &self,
        endpoint: usize,
        ctrl_state: &mut CtrlState,
        status: EndpointStatusValue,
    ) {
        let mut again = true;
        while again {
            again = false;
            // `again` may be set to true below when it would be
            // advantageous to process more flags without waiting for
            // another interrupt.

            match *ctrl_state {
                CtrlState::Init => {
                    if status.is_set(EndpointStatus::RXSTP) {
                        // We received a SETUP transaction

                        debug1!("D({}) RXSTP", endpoint);
                        // self.debug_show_d0();

                        let packet_bytes = self.descriptors[0][0].packet_size.read(PacketSize::BYTE_COUNT);
                        let result = if packet_bytes == 8 {
                            self.client.map(|c| c.ctrl_setup(endpoint))
                        } else {
                            Some(CtrlSetupResult::ErrBadLength)
                        };

                        match result {
                            Some(CtrlSetupResult::Ok) => {
                                // Unsubscribe from SETUP interrupts
                                endpoint_disable_interrupts(endpoint, EndpointControl::RXSTPE::SET);

                                if status.matches_all(EndpointStatus::CTRLDIR::In) {
                                    // The following Data stage will be IN

                                    // Wait until bank is clear to send (TXIN).
                                    // Also wait for NAKOUT to signal end of IN
                                    // stage (the datasheet incorrectly says
                                    // NAKIN).
                                    usbc_regs().uestaclr[endpoint]
                                        .write(EndpointStatus::NAKOUT::SET);
                                    endpoint_enable_interrupts(
                                        endpoint,
                                        EndpointControl::TXINE::SET + EndpointControl::NAKOUTE::SET,
                                    );
                                    *ctrl_state = CtrlState::ReadIn;
                                } else {
                                    // The following Data stage will be OUT

                                    // Wait for OUT packets (RXOUT).  Also wait
                                    // for NAKIN to signal end of OUT stage.
                                    usbc_regs().uestaclr[endpoint]
                                        .write(EndpointStatus::NAKIN::SET);
                                    endpoint_enable_interrupts(
                                        endpoint,
                                        EndpointControl::RXOUTE::SET + EndpointControl::NAKINE::SET,
                                    );
                                    *ctrl_state = CtrlState::WriteOut;
                                }
                            }
                            failure => {
                                // Respond with STALL to any following
                                // transactions in this request
                                usbc_regs().ueconset[endpoint].write(EndpointControl::STALLRQ::SET);

                                match failure {
                                    None => debug1!("D({}) No client to handle Setup", endpoint),
                                    Some(_err) => {
                                        debug1!("D({}) Client err on Setup: {:?}", endpoint, _err)
                                    }
                                }

                                // Remain in Init state for next SETUP
                            }
                        }

                        // Acknowledge SETUP interrupt
                        usbc_regs().uestaclr[endpoint].write(EndpointStatus::RXSTP::SET);
                    }
                }
                CtrlState::ReadIn => {
                    // TODO: Handle Abort as described in 17.6.2.15
                    // (for Control and Isochronous only)

                    if status.is_set(EndpointStatus::NAKOUT) {
                        // The host has completed the IN stage by sending an OUT token

                        endpoint_disable_interrupts(
                            endpoint,
                            EndpointControl::TXINE::SET + EndpointControl::NAKOUTE::SET,
                        );

                        debug1!("D({}) NAKOUT", endpoint);
                        self.client.map(|c| c.ctrl_status(endpoint));

                        // Await end of Status stage
                        endpoint_enable_interrupts(endpoint, EndpointControl::RXOUTE::SET);

                        // Acknowledge
                        usbc_regs().uestaclr[endpoint].write(EndpointStatus::NAKOUT::SET);

                        *ctrl_state = CtrlState::ReadStatus;

                        // Run handler again in case the RXOUT has already arrived
                        again = true;

                    } else if status.is_set(EndpointStatus::TXIN) {

                        // The data bank is ready to receive another IN payload
                        debug1!("D({}) TXIN", endpoint);

                        let result = self.client.map(|c| {
                            // Allow client to write a packet payload to buffer
                            c.ctrl_in(endpoint)
                        });
                        match result {
                            Some(CtrlInResult::Packet(packet_bytes, transfer_complete)) => {
                                self.descriptors[0][0].packet_size.write(if packet_bytes == 8
                                    && transfer_complete
                                {
                                    // Send a complete final packet, and request
                                    // that the controller also send a zero-length
                                    // packet to signal the end of transfer
                                    PacketSize::BYTE_COUNT.val(8) +
                                        PacketSize::AUTO_ZLP::Yes
                                } else {
                                    // Send either a complete but not-final
                                    // packet, or a short and final packet (which
                                    // itself signals end of transfer)
                                    PacketSize::BYTE_COUNT.val(packet_bytes as u32)
                                });

                                debug1!(
                                    "D({}) Send CTRL IN packet ({} bytes)",
                                    endpoint,
                                    packet_bytes
                                );
                                // self.debug_show_d0();

                                if transfer_complete {
                                    // IN data completely sent.  Unsubscribe from TXIN.
                                    // (Continue awaiting NAKOUT to indicate end of Data stage)
                                    endpoint_disable_interrupts(
                                        endpoint,
                                        EndpointControl::TXINE::SET,
                                    );
                                } else {
                                    // Continue waiting for next TXIN
                                }

                                // Clear TXIN to signal to the controller that the IN payload is
                                // ready to send
                                usbc_regs().uestaclr[endpoint].write(EndpointStatus::TXIN::SET);
                            }
                            Some(CtrlInResult::Delay) => {
                                endpoint_disable_interrupts(endpoint, EndpointControl::TXINE::SET);

                                debug1!("*** Client NAK");

                                // XXX set busy bits?

                                *ctrl_state = CtrlState::InDelay;
                            }
                            _ => {
                                // Respond with STALL to any following IN/OUT transactions
                                usbc_regs().ueconset[endpoint].write(EndpointControl::STALLRQ::SET);

                                debug1!("D({}) Client IN err => STALL", endpoint);

                                // Wait for next SETUP
                                endpoint_disable_interrupts(endpoint, EndpointControl::NAKOUTE::SET);
                                endpoint_enable_interrupts(endpoint, EndpointControl::RXSTPE::SET);
                                *ctrl_state = CtrlState::Init;
                            }
                        }
                    }
                }
                CtrlState::ReadStatus => {
                    if status.is_set(EndpointStatus::RXOUT) {
                        // Host has completed Status stage by sending an OUT packet

                        debug1!("D({}) RXOUT: End of Control Read transaction", endpoint);

                        // Wait for next SETUP
                        endpoint_disable_interrupts(endpoint, EndpointControl::RXOUTE::SET);
                        endpoint_enable_interrupts(endpoint, EndpointControl::RXSTPE::SET);
                        *ctrl_state = CtrlState::Init;

                        // Acknowledge
                        usbc_regs().uestaclr[endpoint].write(EndpointStatus::RXOUT::SET);

                        self.client.map(|c| c.ctrl_status_complete(endpoint));
                    }
                }
                CtrlState::WriteOut => {
                    if status.is_set(EndpointStatus::RXOUT) {
                        // Received data

                        debug1!("D({}) RXOUT: Received Control Write data", endpoint);
                        // self.debug_show_d0();

                        // Pass the data to the client and see how it reacts
                        let result = self.client.map(|c| {
                            c.ctrl_out(
                                endpoint,
                                self.descriptors[0][0].packet_size.read(PacketSize::BYTE_COUNT),
                            )
                        });
                        match result {
                            Some(CtrlOutResult::Ok) => {
                                // Acknowledge
                                usbc_regs().uestaclr[endpoint].write(EndpointStatus::RXOUT::SET);
                            }
                            Some(CtrlOutResult::Delay) => {
                                // Don't acknowledge; hardware will have to send NAK

                                // Unsubscribe from RXOUT until client says it is ready
                                // (But there is not yet any interface for that)
                                endpoint_disable_interrupts(endpoint, EndpointControl::RXOUTE::SET);
                            }
                            _ => {
                                // Respond with STALL to any following transactions
                                // in this request
                                usbc_regs().ueconset[endpoint].write(EndpointControl::STALLRQ::SET);

                                debug1!("D({}) Client OUT err => STALL", endpoint);

                                // Wait for next SETUP
                                endpoint_disable_interrupts(endpoint, EndpointControl::RXOUTE::SET +
                                                            EndpointControl::NAKINE::SET);
                                endpoint_enable_interrupts(endpoint, EndpointControl::RXSTPE::SET);
                                *ctrl_state = CtrlState::Init;
                            }
                        }
                    }
                    if status.is_set(EndpointStatus::NAKIN) {
                        // The host has completed the Data stage by sending an IN token
                        debug1!("D({}) NAKIN: Control Write -> Status stage", endpoint);

                        // Acknowledge
                        usbc_regs().uestaclr[endpoint].write(EndpointStatus::NAKIN::SET);

                        // Wait for bank to be free so we can write ZLP to acknowledge transfer
                        endpoint_disable_interrupts(
                            endpoint,
                            EndpointControl::RXOUTE::SET + EndpointControl::NAKINE::SET,
                        );
                        endpoint_enable_interrupts(endpoint, EndpointControl::TXINE::SET);
                        *ctrl_state = CtrlState::WriteStatus;

                        // Can probably send the ZLP immediately
                        again = true;
                    }
                }
                CtrlState::WriteStatus => {
                    if status.is_set(EndpointStatus::TXIN) {
                        debug1!(
                            "D({}) TXIN for Control Write Status (will send ZLP)",
                            endpoint
                        );

                        self.client.map(|c| c.ctrl_status(endpoint));

                        // Send zero-length packet to acknowledge transaction
                        self.descriptors[0][0]
                            .packet_size
                            .write(PacketSize::BYTE_COUNT.val(0));

                        // Signal to the controller that the IN payload is ready to send
                        usbc_regs().uestaclr[endpoint].write(EndpointStatus::TXIN::SET);

                        // Wait for TXIN again to confirm that IN payload has been sent
                        *ctrl_state = CtrlState::WriteStatusWait;
                    }
                }
                CtrlState::WriteStatusWait => {
                    if status.is_set(EndpointStatus::TXIN) {
                        debug1!("D({}) TXIN: Control Write Status Complete", endpoint);

                        // Wait for next SETUP
                        endpoint_enable_interrupts(endpoint, EndpointControl::RXSTPE::SET);
                        *ctrl_state = CtrlState::Init;

                        // for SetAddress, client must enable address after STATUS stage
                        self.client.map(|c| c.ctrl_status_complete(endpoint));
                    }
                }
                CtrlState::InDelay =>  { /* XXX: Spin fruitlessly */ }
            }

            // Uncomment the following line to run the above while loop only once per interrupt,
            // which can make debugging easier.
            //
            // again = false;
        }
    }

    fn handle_bulk_endpoint_interrupt(
        &self,
        endpoint: usize,
        bulk_state: &mut BulkState,
        status: EndpointStatusValue,
    ) {
        match *bulk_state {
            BulkState::Init => {
                if status.is_set(EndpointStatus::TXIN) {
                    // We got an IN request from the host

                    debug1!("D({}) TXIN", endpoint);

                    // Acknowledge
                    usbc_regs().uestaclr[endpoint].write(EndpointStatus::TXIN::SET);

                    if !usbc_regs().uecon[endpoint].is_set(EndpointControl::FIFOCON) {
                       debug!("Got TXIN but not FIFOCON");
                       return;
                    }
                    // A bank is free to write an IN packet

                    // Write data
                    let result = self.client.map(|c| {
                        // Allow client to write a packet payload to buffer
                        c.bulk_in(endpoint)
                    });
                    match result {
                        Some(BulkInResult::Packet(packet_bytes, transfer_complete)) => {

                            self.descriptors[0][0].packet_size.write(if packet_bytes == 8
                                && transfer_complete
                            {
                                // Send a complete final packet, and request
                                // that the controller also send a zero-length
                                // packet to signal the end of transfer
                                PacketSize::BYTE_COUNT.val(8) +
                                    PacketSize::AUTO_ZLP::Yes
                            } else {
                                // Send either a complete but not-final
                                // packet, or a short and final packet (which
                                // itself signals end of transfer)
                                PacketSize::BYTE_COUNT.val(packet_bytes as u32)
                            });

                            // Clear FIFOCON to signal data ready to send
                            usbc_regs().ueconclr[endpoint].write(EndpointControl::FIFOCON::SET);

                            debug1!(
                                "D({}) Send BULK IN packet ({} bytes)",
                                endpoint,
                                packet_bytes
                            );

                            // Remain in Init state
                        }
                        Some(BulkInResult::Delay) => {
                            // The client is not ready to send data

                            endpoint_disable_interrupts(endpoint, EndpointControl::TXINE::SET);

                            *bulk_state = BulkState::InDelay;
                        }
                        _ => {
                            debug1!("D({}) Client IN err => STALL", endpoint);

                            // Respond with STALL to any following IN/OUT transactions
                            usbc_regs().ueconset[endpoint].write(EndpointControl::STALLRQ::SET);

                            // XXX: interrupts?

                            // Remain in Init state?
                        }
                    }
                }
            }
            BulkState::InDelay => { /* XXX: Spin fruitlessly */ }
        }
    }

    #[allow(dead_code)]
    fn debug_show_d0(&self) {
        for bi in 0..1 {
            let b = &self.descriptors[0][bi];
            let addr = b.addr.get();
            let _buf = if addr.is_null() {
                None
            } else {
                unsafe {
                    Some(slice::from_raw_parts(
                        addr,
                        b.packet_size.read(PacketSize::BYTE_COUNT) as usize,
                    ))
                }
            };

            debug1!(
                "B_0_{} \
                 \n     {:?}\
                 \n     {:?}\
                 \n     {:?}",
                bi, // (&b.addr as *const _), b.addr.get(),
                b.packet_size.get(),
                b.control_status.get(),
                _buf.map(HexBuf)
            );
        }
    }

    pub fn mode(&self) -> Option<Mode> {
        match self.get_state() {
            State::Idle(mode) => Some(mode),
            State::Active(mode) => Some(mode),
            _ => None,
        }
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

    // TODO: Remote wakeup (Device -> Host, after receiving DEVICE_REMOTE_WAKEUP)
}

#[inline]
fn endpoint_disable_interrupts(endpoint: usize, mask: FieldValue<u32, EndpointControl::Register>) {
    usbc_regs().ueconclr[endpoint].write(mask);
}

#[inline]
fn endpoint_enable_interrupts(endpoint: usize, mask: FieldValue<u32, EndpointControl::Register>) {
    usbc_regs().ueconset[endpoint].write(mask);
}

impl<'a> UsbController for Usbc<'a> {
    fn endpoint_set_buffer<'b>(&'b self, endpoint: usize, buf: &[VolatileCell<u8>]) {
        if buf.len() != 8 {
            client_err!("Bad endpoint buffer size");
        }

        self._endpoint_bank_set_buffer(EndpointIndex::new(endpoint), BankIndex::Bank0, buf);
    }

    fn enable_as_device(&self, speed: DeviceSpeed) {
        let speed = match speed {
            DeviceSpeed::Full => Speed::Full,
            DeviceSpeed::Low => Speed::Low,
        };

        match self.get_state() {
            State::Reset => self._enable(
                Mode::Device {
                    speed: speed,
                    config: Default::default(),
                    state: Default::default(),
                }),
            _ => client_err!("Already enabled"),
        }
    }

    fn attach(&self) {
        match self.get_state() {
            State::Reset => client_warn!("Not enabled"),
            State::Active(_) => client_warn!("Already attached"),
            State::Idle(_) => self._attach(),
        }
    }

    fn detach(&self) {
        match self.get_state() {
            State::Reset => client_warn!("Not enabled"),
            State::Idle(_) => client_warn!("Not attached"),
            State::Active(_) => self._detach(),
        }
    }

    fn endpoint_ctrl_out_enable(&self, endpoint: usize) {
        let endpoint_cfg = RegisterValue::new(From::from(
                                EndpointConfig::EPTYPE::Control +
                                EndpointConfig::EPDIR::Out +
                                EndpointConfig::EPSIZE::Bytes8 +
                                EndpointConfig::EPBK::Single));

        if match self.get_state() {
            State::Reset => client_err!("Not enabled"),
            State::Idle(Mode::Device { .. }) => {
                // The endpoint will be active when we attach
                true
            }
            State::Active(Mode::Device { .. }) => {
                // The endpoint will be active immediately
                true
            }
            _ => client_err!("Not in Device mode"),
        } {
            self._endpoint_enable(endpoint, endpoint_cfg)
        }
    }

    fn set_address(&self, addr: u16) {
        usbc_regs()
            .udcon
            .modify(DeviceControl::UADD.val(addr as u32));

        debug1!("Set Address = {}", addr);
    }

    fn enable_address(&self) {
        usbc_regs().udcon.modify(DeviceControl::ADDEN::SET);

        debug1!(
            "Enable Address = {}",
            usbc_regs().udcon.read(DeviceControl::UADD)
        );
    }
}

/// Static state to manage the USBC
pub static mut USBC: Usbc<'static> = Usbc::new();
