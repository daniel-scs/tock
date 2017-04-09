
pub trait Client {
    fn received_setup(&self /* , descriptor/bank */);

    fn received_out(&self /* , descriptor/bank */);
}
