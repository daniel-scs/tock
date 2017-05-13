//! Interface to USB controller hardware

use common::volatile_slice::VolatileSlice;

/// USB controller interface
pub trait UsbController {
    fn enable_device(&self, full_speed: bool);

    fn attach(&self);

    fn endpoint_set_buffer(&self, e: u32, buf: VolatileSlice<u8>);

    fn endpoint_ctrl_out_enable(&self, e: u32);

    fn set_address(&self, addr: u16);

    fn enable_address(&self);
}

/// USB controller client interface
pub trait Client {
    fn enable(&self);
    fn attach(&self);
    fn bus_reset(&self);

    fn ctrl_setup(&self) -> bool;
    fn ctrl_in(&self) -> CtrlInResult;
    fn ctrl_out(&self /* , descriptor/bank */) {}
    fn ctrl_status(&self) {}
    fn ctrl_status_complete(&self) {}
}

pub enum CtrlInResult {
    Packet(usize, bool),
    Delay,
    Error,
}
