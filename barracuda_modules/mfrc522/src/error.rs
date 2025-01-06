
#[derive(Debug)]
pub enum TxpError
{
    _NoTxp,
    GeneralError,
    Timeout,
    CommunicationLost,
    UnsupportedTagType,
    ChipError(u8)
}