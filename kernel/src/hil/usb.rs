
pub trait Client {
    pub fn received_setup(&self, /* descriptor/bank */);

    pub fn received_out(&self, /* descriptor/bank */);
}
