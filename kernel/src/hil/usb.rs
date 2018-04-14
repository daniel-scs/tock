//! Interface to USB controller hardware

use common::VolatileCell;

/// USB controller interface
pub trait UsbController {
    // Should be called before `enable_as_device()`
    fn endpoint_set_buffer(&self, endpoint: u32, buf: &[VolatileCell<u8>]);

    // Must be called before `attach()`
    fn enable_as_device(&self, speed: DeviceSpeed);

    fn attach(&self);

    fn detach(&self);

    fn endpoint_ctrl_out_enable(&self, endpoint: u32);

    fn set_address(&self, addr: u16);

    fn enable_address(&self);
}

/// USB controller client interface
pub trait Client {
    fn enable(&self);
    fn attach(&self);
    fn bus_reset(&self);

    fn ctrl_setup(&self) -> CtrlSetupResult;
    fn ctrl_in(&self) -> CtrlInResult;
    fn ctrl_out(&self, packet_bytes: u32) -> CtrlOutResult;
    fn ctrl_status(&self);
    fn ctrl_status_complete(&self);
}

pub enum DeviceSpeed {
    Full,
    Low,
}

#[derive(Debug)]
pub enum CtrlSetupResult {
    /// The Setup request was handled successfully
    Ok,

    // The Setup request cannot be handled; abort this transfer with STALL
    ErrBadLength,
    ErrNoParse,
    ErrNonstandardRequest,
    ErrUnrecognizedDescriptorType,
    ErrUnrecognizedRequestType,
    ErrNoDeviceQualifier,
    ErrInvalidDeviceIndex,
    ErrInvalidConfigurationIndex,
    ErrInvalidStringIndex,
}

pub enum CtrlInResult {
    /// A packet of the given size was written into the endpoint buffer
    Packet(usize, bool),

    /// The client is not yet able to provide data to the host, but may
    /// be able to in the future.  This result causes the controller
    /// to send a NAK token to the host.
    Delay,

    /// The client does not support the request.  This result causes the
    /// controller to send a STALL token to the host.
    Error,
}

pub enum CtrlOutResult {
    /// Data received (send ACK)
    Ok,

    /// Not ready yet (send NAK)
    Delay,

    /// In halt state (send STALL)
    Halted,
}
