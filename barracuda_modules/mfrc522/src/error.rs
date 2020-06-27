
#[derive(Debug)]
pub enum TxpError
{
    NoTxp,
    GeneralError,
    Timeout,
    CommunicationLost,
    UnsupportedTagType,
    ChipError(u8)
}