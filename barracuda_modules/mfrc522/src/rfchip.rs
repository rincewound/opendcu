
use crate::error::TxpError;

pub trait RFChip
{
    fn send_picc(&self, data: Vec<u8>) -> Result<Vec<u8>, TxpError>;
    fn toggle_bit_framing(&self, enable: bool);
}
