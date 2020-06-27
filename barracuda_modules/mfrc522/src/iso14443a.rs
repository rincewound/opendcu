
use crate::{error::TxpError, rfchip::RFChip};
use std::iter::FromIterator;

#[allow(dead_code)]
pub enum Iso14443aCommand
{
    ReqA                = 0x26,     // AKA REQIDL
    ReqAll              = 0x52,
    AnticollCasc1       = 0x93,     // is also select_tag in original.
    AnticollCasc2       = 0x95,
    AnticollCasc3       = 0x97,
    Authent1A           = 0x60,
    Authent1B           = 0x61,
    Read                = 0x30,
    Write               = 0xA0,
    Decrement           = 0xC0,
    Increment           = 0xC1,
    Restore             = 0xC2,
    Transfer            = 0xB0,
    Halt                = 0x50 
}

const sak_uid_not_complete_mask: u8 = 0b00000100;
const sak_uid_complete_mask: u8 = 0b00100000;

pub struct Iso14443A<'a,T> where T: RFChip
{
    rf_chip: &'a T
}


impl<'a, T:RFChip> Iso14443A<'a, T>
{
    pub fn new(chip: &'a T) -> Self
    {
        Self{
            rf_chip: chip
        }
    }

    fn do_picc_command(&self, cmd: Iso14443aCommand, data: Option<Vec<u8>>) -> Result<Vec<u8>, TxpError>
    {
        let mut cmd = vec![cmd as u8];
        if let Some(mut payload) = data
        {
            cmd.append(&mut payload);
        }
        return self.rf_chip.send_picc(cmd);
    }

    pub fn search_txp(&self) -> Result<Vec<u8>, TxpError>
    {
        let atqa = self.do_picc_command(Iso14443aCommand::ReqA, None)?;
        // We should have received
        // an ATQA response containing at least the UID size of a txp, if
        // present.
        if atqa.len() != 2
        {
            return Err(TxpError::GeneralError);
        }

        let uid = self.txp_anticoll()?;
        Ok(uid)
    }

    fn check_bcc(&self, data: &[u8]) -> Result<(), TxpError>
    {
        let mut bcc: u8 = 0x00;
        for idx in 0..data.len() - 1
        {
            bcc = bcc ^ data[idx]
        }

        if bcc != data[data.len() - 1]
        {
            return Err(TxpError::CommunicationLost);
        }
        return Ok(());
    }

    fn txp_anticoll(&self)-> Result<Vec<u8>, TxpError>
    {
        //self.write_mfrc522(ChipRegisters::BitFramingReg as u8, &[0x00 as u8]);
        self.rf_chip.toggle_bit_framing(false);

        // The 0x20 is actually the NVB!
        let mut res = self.do_picc_command( Iso14443aCommand::AnticollCasc1, Some(vec![0x20]))?;

        // The anti collision loop should go here... but alas:
        // Note, that we do not really support anti coll here, but we use the anticoll
        // procedure as specified for ISO14443A Tags to obtain the fullsize UID.
        if res.len() != 5
        {
            return Err(TxpError::UnsupportedTagType);
        }

        // The last byte of the data contains an XOR blockcheck.
        // this needs to be done in case the reception was interrupted
        // and we only got parts of the UID.
        let _ = self.check_bcc(&res)?;

        // check if this UID is complete (see 8371.TypeA_UID_retrieval.pdf)
        if res[0] != 0x88
        {
            return Ok(Vec::from_iter(res[0..4].iter().cloned()))
        }
        
        let mut select_data = vec![0x70];
        // We want to retrieve the rest of the UID, so the bits
        // already received are the prefix.
        select_data.append(&mut res.clone());
        let sak = self.do_picc_command(Iso14443aCommand::AnticollCasc1, Some(select_data))?[0];
        if sak != sak_uid_not_complete_mask
        {
            // SAK states UID is ncomplete (i.e. != 0x04)
            // in this case the previously received magic
            // 0x88 byte is actually part of the UID.
            return Ok(Vec::from_iter(res[0..4].iter().cloned()))
        }

        let uid2 = self.do_picc_command( Iso14443aCommand::AnticollCasc2, Some(vec![0x20 as u8]))?;
        let _ = self.check_bcc(&uid2)?;

        // check if this UID is complete (see 8371.TypeA_UID_retrieval.pdf)
        if uid2[0] != 0x88
        {
            res.extend_from_slice(&uid2[0..3]);
            return Ok(Vec::from_iter(res[0..7].iter().cloned()))
        }

        // We can't deal with 10 byte UIDs yet.
        return Err(TxpError::UnsupportedTagType);

        // Note: This part deals with 10 byte uids.
        // let mut seven_uid_bytes = vec<u8>::new();
        // seven_uid_bytes.extend_from_slice(&res[0..4]);
        // seven_uid_bytes.extend_from_slice(&uid2[1..4]);

        // select_data = vec![Iso1443aCommand::AnticollCasc3 as u8, 0x70];
        // // We want to retrieve the rest of the UID, so the bits
        // // already received are the prefix.
        // select_data.append(&mut res.clone());

        // let uid3 = self.send_picc_command( &[Iso1443aCommand::AnticollCasc3 as u8, 0x20])?;
        // let _ = self.check_bcc(&uid3)?;
        // res.extend_from_slice(&uid3[0..3]);
        
        // return Ok(Vec::from_iter(res[0..9].iter().cloned()))
    }
}

#[cfg(test)]
mod tests {

    use crate::{error::TxpError, rfchip::*};
    use mockall::{automock, mock, predicate::*};
    use super::{Iso14443aCommand, Iso14443A};
    
    #[test]
    fn search_txp_sends_reqa() 
    {
        let mut mock = MockRFChip::new();
        mock.expect_send_picc()
            .with(eq(vec![Iso14443aCommand::ReqA as u8]))
            .returning(|x| Err(TxpError::Timeout));
        let iso = Iso14443A::new(&mock);
        iso.search_txp();
    }
}