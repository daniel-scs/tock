use returncode::ReturnCode;

pub trait CRC {
    fn get_version(&self) -> u32;
    fn compute(&self, data: &[u8]) -> ReturnCode;
}

pub trait Client {
    fn receive_result(&self, u32);
}
