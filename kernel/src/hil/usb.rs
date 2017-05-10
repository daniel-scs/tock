//! Interface to USB controller hardware

pub trait Client {
    fn bus_reset(&self);

    fn received_setup_in(&self, setup_data: &[u8]) -> InRequestResult;

    fn received_out(&self /* , descriptor/bank */);
}

pub enum InRequestResult {
    Error,
    Data(&'static [u8]),
    // Delay,
}
