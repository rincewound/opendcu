
use crate::error::TxpError;

#[cfg(test)]
use mockall::{automock, predicate::*};
#[cfg_attr(test, automock)]
pub trait RFChip
{
    fn send_picc(&self, data: Vec<u8>) -> Result<Vec<u8>, TxpError>;
    fn toggle_bit_framing(&self, enable: bool);
}
