
pub trait Client {
    fn bus_reset(&self);

    fn received_setup(&self /* , descriptor/bank */);

    fn received_out(&self /* , descriptor/bank */);
}
