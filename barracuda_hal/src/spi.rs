pub trait SpiInterface
{
    fn send_receive(&self, data: &[u8]) -> Vec<u8>;
}