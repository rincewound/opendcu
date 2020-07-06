/*
    # The ISO 14443A Implementation

    This module implements an compliant ISO14443A 
    protocol stack that can be used to read
    smart cards or to communicate with NFC devices.


    ## Prerequesites
    The protocol implementation needs to be able to
    send and receive data via RF to a transponder.
    As such it requires an implementation of the
    RFChip trait. 
  
    ## Searching a transponder

    The ISO/IEC 14443 specifies that cards following the 
    ISO/IEC 14443A shall not interfere cards following 
    the ISO/IEC 14443B, and vice versa. In any case, 
    the card activation procedure starts with a 
    Request command (REQA or REQB), which is used only 
    to check whether there is at least one card in the 
    reader field. The REQA or REQB has to be sent after 
    the carrier is switched on, waiting 5 ms at minimum 
    before starting the transmission.

    For NFC devices, there has to be another block between 
    “Card Polling” and “Switch on RF”, because NFC devices 
    need to check whether there is already a field available 
    or not. If an external field is detected, the reader 
    is not allowed to switch on its own RF field.

    Command Order for searching a transponder (14443a only!)

    * Enable RF
        * Delay >= 5ms
        * Send REQA/REQIDL (0x23)
        * If not ATQA received: Terminate
            [* Activate Card]
            [* Perform transaction]
            [* Halt/Deselect]
    * RF Off


    ### Activating a card
    Card activation will yield the UID of a given medium,
    procedure:
    * Do Anticollision
    * Check SAK Bit 6 == 1, if not: Terminate
    * RATS + PSS if required
    * Card is now selected


    ### Anticollision Loop


*/

use crate::{error::TxpError, rfchip::RFChip};
use std::iter::FromIterator;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[allow(dead_code)]
pub enum Iso14443aCommand
{
    ReqA                = 0x26,     // AKA REQIDL
    ReqAll              = 0x52,     // AKA WUPA
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


/// # The NXP/Mifare Txp type 
/// Most transponders can sort-of be identified by the
/// SAK they transmit (see NXP AN10834). We use this information
/// to give out a hint at the transpondertype the chip is up
/// against.
/// Note that these are *hints* at best and ambigious in most
/// cases. However they should suffice to detect unsafe, old
/// transponders (i.e. Mifare Classic)
#[repr(u8)]
#[derive(Debug, Copy, Clone, TryFromPrimitive)]
pub enum TxpType
{
    /// Returned by Mifare Transponders, that have not been through
    /// the anti-coll loop yet.
    AnyMifare = 0x04,
    /// Any mifare ultralight transponder
    Ultralight = 0x00,
    /// Any mifare mini transponder
    Mini = 0x09,
    /// Classic or Mifare Plus transponders with max 2 KiB of storage.
    ClassicOrPlusMax2k = 0x08,
    /// Classic or Mifare Plus transponders with max 4 KiB of storage.
    ClassicOrPlusMax4k = 0x18,
    /// Mifare Plus Level 2 Transponders with max 2 KiB of storage.
    Plus2kL2 = 0x10,
    /// Mifare Plus Level 2 Transponders with max 4 KiB of storage.
    Plus4L2 = 0x11,
    /// Mifare Plus Level 3 Transponders
    AnyPlusL3 = 0x20,
    /// Mifare Desfire Transponders
    AnyDesfire = 0x24,
}

#[derive(Debug, Clone)]
pub struct Iso14443aTransponder
{
    pub txpType: TxpType,
    pub uid: Vec<u8>,    
}


impl Iso14443aTransponder
{
    pub fn new(txpType: TxpType, uid: Vec<u8>) -> Self
    {
        Self {txpType, uid}
    }

    pub fn from_u8(txpType: u8, uid: Vec<u8>) -> Self
    {
        Self {txpType: TxpType::try_from(txpType).unwrap(), uid}
    }
}

const increase_cascade_mask: u8 = 0b00000100;
//const increase_cascade_mask: u8 = 0b00100000;

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

    pub fn search_txp(&self) -> Result<Iso14443aTransponder, TxpError>
    {
        let atqa = self.do_picc_command(Iso14443aCommand::ReqA, None)?;
        // We should have received
        // an ATQA response.
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

    fn txp_anticoll(&self)-> Result<Iso14443aTransponder, TxpError>
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

        // Note: This case *should* be covered by the handling of the SAK further down.
        // check if this UID is complete (see 8371.TypeA_UID_retrieval.pdf)
        // if res[0] != 0x88
        // {
        //     return Ok(Iso14443aTransponder::new(0, Vec::from_iter(res[0..4].iter().cloned())))
        // }
        
        let mut select_data = vec![0x70];
        // We want to retrieve the rest of the UID, so the bits
        // already received are the prefix.
        select_data.append(&mut res.clone());
        let sak = self.do_picc_command(Iso14443aCommand::AnticollCasc1, Some(select_data))?[0];
        if sak & increase_cascade_mask != increase_cascade_mask
        {
            // SAK states UID is incomplete (i.e. != 0x04)
            // in this case the previously received magic
            // 0x88 byte is actually part of the UID.
            return Ok(Iso14443aTransponder::from_u8(sak, Vec::from_iter(res[0..4].iter().cloned())))
        }

        let uid2 = self.do_picc_command( Iso14443aCommand::AnticollCasc2, Some(vec![0x20 as u8]))?;
        let _ = self.check_bcc(&uid2)?;

        // Get select ackknowledge
        let mut sak2selectdata = vec![0x70];
        sak2selectdata.extend_from_slice(&uid2[..]);            
        let sak2 = self.do_picc_command( Iso14443aCommand::AnticollCasc2, Some(sak2selectdata))?[0];
        
        if sak2 & increase_cascade_mask == increase_cascade_mask
        {
            // We can't deal with 10 byte UIDs yet.
            return Err(TxpError::UnsupportedTagType);
        }

        res.pop();  // this is the bcc that still floats in res
        res.extend_from_slice(&uid2[0..4]);
        return Ok(Iso14443aTransponder::from_u8(sak2, Vec::from_iter(res[1..8].iter().cloned())))


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
        let _= iso.search_txp();
    }

    #[test]
    fn search_txp_yields_4byte_uid_if_uid_is_complete()
    {
        let mut mock = MockRFChip::new();
        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::ReqA as u8]))
        .returning(|x| Ok(vec![0xAB, 0x04]));

        mock.expect_toggle_bit_framing()
        .with(eq(false)).return_const(());

        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::AnticollCasc1 as u8, 0x20]))
        .returning(|x| Ok(vec![0xAB, 0x04, 0xDA, 0xE9, 0x9C]));

        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::AnticollCasc1 as u8, 0x70, 0xAB, 0x04, 0xDA, 0xE9, 0x9C]))
        .returning(|_| Ok(vec![0x00]));        

        let iso = Iso14443A::new(&mock);
        let result = iso.search_txp();
        assert!(result.is_ok());
    }

    
    #[test]
    fn search_txp_yields_7byte_uid_if_uid_is_complete()
    {
        let mut mock = MockRFChip::new();
        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::ReqA as u8]))
        .returning(|_| Ok(vec![0xAB, 0x04]));

        mock.expect_toggle_bit_framing()
        .with(eq(false)).return_const(());

        // Indicate incomplete UID using 0x88
        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::AnticollCasc1 as u8, 0x20]))
        .returning(|_| Ok(vec![0x88, 0x04, 0xDA, 0xE9, 0xBF]));

        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::AnticollCasc1 as u8, 0x70, 0x88, 0x04, 0xDA, 0xE9, 0xBF]))
        .returning(|_| Ok(vec![0x4]));

        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::AnticollCasc2 as u8, 0x20]))
        .returning(|_| Ok(vec![0xCA, 0xB5, 0x28, 0x80, 0xD7]));
    
        mock.expect_send_picc()
        .with(eq(vec![Iso14443aCommand::AnticollCasc2 as u8, 0x70, 0xCA, 0xB5, 0x28, 0x80, 0xD7]))
        .returning(|_| Ok(vec![0x0]));

        let iso = Iso14443A::new(&mock);
        let result = iso.search_txp();
        assert!(result.is_ok());
        let uid = result.unwrap();
        assert_eq!(&[0x4, 0xDA, 0xE9, 0xCA, 0xB5, 0x28, 0x80], &uid.uid[..]);
    }

    
    #[test]
    fn search_txp_yields_10byte_uid_if_uid_is_complete()
    {
        assert!(false)
    }
}