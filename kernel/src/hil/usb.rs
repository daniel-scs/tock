//! Interface to USB controller hardware

pub trait Client {
    fn bus_reset(&self);

    fn received_setup_in(&self, setup_data: &[u8]) -> InRequestResult;

    fn ctrl_in(&self, packet_buf: &mut [u8]) -> CtrlInResult;

    fn received_out(&self /* , descriptor/bank */);
}

pub enum InRequestResult {
    Ok,
    Error,
    // Delay,
}

pub enum CtrlInResult {
    Filled(usize),
    Error,
}
