
/*
    # The MFRC522 ISO 14443A Interface
  
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

use barracuda_hal::{interrupt::Interrupt, spi::SpiInterface};
use std::{time, thread};
use std::iter::FromIterator;

#[allow(dead_code)]
#[derive(Debug,PartialEq, Clone, Copy)]
pub enum ChipCommand
{
    IDLE             = 0x00,
    MEM              = 0x01,
    GenerateRandomId = 0x02,
    TRANSMIT         = 0x04,
    NoCmdChange      = 0x07,
    CALCCRC          = 0x03,    
    RECEIVE          = 0x08,    
    TRANSCEIVE       = 0x0C,        // Send OTA data to txp
    AUTHENT          = 0x0E,
    RESETPHASE       = 0x0F,
    
}

#[allow(dead_code)]
pub enum IrqSources
{
    // Triggered when the internal timer overflows
    TimerIEn    = 0x01,

    // An error occured -> see error register
    ErrIEn      = 0x02,

    LoAlertIEn  = 0x04,
    HiAlertIEn  = 0x08,
    // Triggered when a command terminates
    IdleIEn     = 0x10,
    // End of receive
    RxIEn       = 0x20,
    // End of Transmit
    TxIEn       = 0x40,

    IRqInv      = 0x80
}


#[allow(dead_code)]
pub enum ErrorRegBits
{    
    ProtocolError   = 0x01,
    ParityError     = 0x02,
    CRCError        = 0x04, // RxModeReg RxCRCEn bit is set and the CRC calculation fails
                            // automatically cleared to logic 0 during receiver start-up phase
    CollisionError  = 0x08, // Generated during anticoll
    BufferOverflow  = 0x10, // Generated on FIFO Reg overflow
    RFU             = 0x20,
    Temperature     = 0x30, // Chip is overheating. Antenna was turned off
    WriteErr        = 0x40, // data is written into the FIFO buffer by the 
                            // host during the MFAuthent command or if data 
                            // is written into the FIFO buffer by the host 
                            // during the time between sending the last bit 
                            // on the RF interface and receiving the last bit 
                            // on the RF interface
}

#[allow(dead_code)]
enum StatusRegister1Bits
{
    RFU2            = 0x80,
    CrcOk           = 0x40, // Crc Result is zero
    CrcReady        = 0x20, // CRC Calculation has finished.
    IrqActive       = 0x10, // Set that some kind of IRQ was triggered, check ComIEnReg
    TimerRunning    = 0x08, // Set if the timerunit is currently running
    RFU1            = 0x04,
    HiAlert         = 0x02,
    LoAlert         = 0x01
}

#[allow(dead_code)]
pub enum StatusRegister2Bits
{
    TempSensClear   = 0x80,
    I2CForcesHS     = 0x40,
    RFU             = 0x30, // Bits 4 and 5
    MFCrypto1On     = 0x08,
    ModemState      = 0x07  // bits 0 to 2
}

#[allow(dead_code)]
pub enum ModemStates
{
    Receiving       = 0x06,
    WaitForData     = 0x05,
    RxWait          = 0x04,
    Transmitting    = 0x03,
    TxWait          = 0x02,
    WaitStartSend   = 0x01,
    Idle            = 0x00
}

#[allow(dead_code)]
pub enum Iso1443aCommand
{
    ReqA                = 0x26,     // AKA REQIDL
    ReqAll              = 0x52,
    AnticollCasc1      = 0x93,     // is also select_tag in original.
    AnticollCasc2      = 0x95,
    AnticollCasc3      = 0x97,
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

#[allow(dead_code)]
enum ChipRegisters
{
    Reserved00 = 0x00,
    CommandReg = 0x01,
    CommIEnReg = 0x02,
    DivlEnReg = 0x03,
    CommIrqReg = 0x04,
    DivIrqReg = 0x05,
    ErrorReg = 0x06,
    Status1Reg = 0x07,
    Status2Reg = 0x08,
    FIFODataReg = 0x09,
    FIFOLevelReg = 0x0A,
    WaterLevelReg = 0x0B,
    ControlReg = 0x0C,
    BitFramingReg = 0x0D,
    CollReg = 0x0E,
    Reserved01 = 0x0F,

    Reserved10 = 0x10,
    ModeReg = 0x11,
    TxModeReg = 0x12,
    RxModeReg = 0x13,
    TxControlReg = 0x14,
    TxAutoReg = 0x15,
    TxSelReg = 0x16,
    RxSelReg = 0x17,
    RxThresholdReg = 0x18,
    DemodReg = 0x19,
    Reserved11 = 0x1A,
    Reserved12 = 0x1B,
    MifareReg = 0x1C,
    Reserved13 = 0x1D,
    Reserved14 = 0x1E,
    SerialSpeedReg = 0x1F,

    Reserved20 = 0x20,
    CRCResultRegM = 0x21,
    CRCResultRegL = 0x22,
    Reserved21 = 0x23,
    ModWidthReg = 0x24,
    Reserved22 = 0x25,
    RFCfgReg = 0x26,
    GsNReg = 0x27,
    CWGsPReg = 0x28,
    ModGsPReg = 0x29,
    TModeReg = 0x2A,
    TPrescalerReg = 0x2B,
    TReloadRegH = 0x2C,
    TReloadRegL = 0x2D,
    TCounterValueRegH = 0x2E,
    TCounterValueRegL = 0x2F,

    Reserved30 = 0x30,
    TestSel1Reg = 0x31,
    TestSel2Reg = 0x32,
    TestPinEnReg = 0x33,
    TestPinValueReg = 0x34,
    TestBusReg = 0x35,
    AutoTestReg = 0x36,
    VersionReg = 0x37,
    AnalogTestReg = 0x38,
    TestDAC1Reg = 0x39,
    TestDAC2Reg = 0x3A,
    TestADCReg = 0x3B,
    Reserved31 = 0x3C,
    Reserved32 = 0x3D,
    Reserved33 = 0x3E,
    Reserved34 = 0x3F,
}

#[allow(dead_code)]
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

const sak_uid_not_complete_mask: u8 = 0b00000100;
const sak_uid_complete_mask: u8 = 0b00100000;

pub struct Mfrc522<T, Irq>
    where T: SpiInterface, Irq: Interrupt
{
    spi_interface: T,
    tx_rdy_irq: Irq
}

impl<T, Irq> Mfrc522<T, Irq>
where T: SpiInterface, Irq: Interrupt 
{
    pub fn new(spi: T, irq: Irq) -> Self
    {
        let result = Self 
        {
            spi_interface: spi,
            tx_rdy_irq: irq
        };

        result.write_register(ChipRegisters::TModeReg, 0x8D);
        result.write_register(ChipRegisters::TPrescalerReg, 0x3E);
        result.write_register(ChipRegisters::TxAutoReg, 0x40);
        result.write_register(ChipRegisters::TReloadRegL, 30);
        result.write_register(ChipRegisters::TReloadRegH, 0);
        result.write_register(ChipRegisters::ModeReg, 0x3D);
        
        println!("MFRC Firmwareversion Version: {}", result.read_register(ChipRegisters::VersionReg));

        result
    }

    fn write_mfrc522(&self, address: u8, data: &[u8])
    {
        let mut data_bytes = vec![(address << 1) & 0x7E];
        for i in data
        {
            data_bytes.push(*i);
        }

        let _ = self.spi_interface.send_receive(&data_bytes);
    }

    fn read_mrfrc522(&self, address: u8) -> u8
    {
        let data_bytes = vec![((address <<1) & 0x7E) | 0x80, 0];
        let received = self.spi_interface.send_receive(&data_bytes);
        return received[1];
    }

    fn do_command(&self, command: ChipCommand)
    {
        let cmd = [command as u8];
        self.write_mfrc522(ChipRegisters::CommandReg as u8, &cmd);
    }

    fn write_byte(&self, address: u8, byte: u8)
    {
        let data = [byte];
        self.write_mfrc522(address, &data);
    }

    fn read_register(&self, register: ChipRegisters) -> u8
    {
        return self.read_mrfrc522(register as u8);
    }

    fn write_register(&self, register: ChipRegisters, value: u8)
    {
        return self.write_byte(register as u8, value);
    }

    #[allow(dead_code)]
    pub fn reset(&self)
    {
        self.do_command(ChipCommand::RESETPHASE);
    }

    fn set_bit(&self, address: u8, mask: u8)
    {
        let current_value = self.read_mrfrc522(address);
        self.write_byte(address, current_value | mask);
    }

    fn clear_bit(&self, address: u8, mask: u8)
    {
        let current_value = self.read_mrfrc522(address);
        self.write_byte(address, current_value & (!mask));
    }

    pub fn toggle_antenna(&self, enable_antenna: bool)
    {
        let tx_ctrl = self.read_register(ChipRegisters::TxControlReg);
        if enable_antenna
        {
            if (tx_ctrl & 0x03) != 0x03
            {
                self.set_bit(ChipRegisters::TxControlReg as u8, 0x03);
                // as per spec we need to wait at least 5 ms after enabling RF,
                // before Txps will respond to commands.
                thread::sleep(time::Duration::from_millis(5));
            }
        }
        else
        {
            self.clear_bit(ChipRegisters::TxControlReg as u8, 0x03);
        }
    }

    fn enable_interrupt(&self, irqmask: u8)
    {
        self.set_bit(ChipRegisters::CommIEnReg as u8, irqmask | 0x80);
    }

    fn clear_irq_bits(&self)
    {
        // 0b0111111 *should* clear all IRQ requests.
        self.write_register(ChipRegisters::CommIrqReg, 0x7F);
    }

    fn clear_fifo(&self)
    {
        self.write_register(ChipRegisters::FIFOLevelReg, 0x80);
    }

    fn handle_error(&self) -> Result<(), TxpError>
    {
        let error = self.read_register(ChipRegisters::ErrorReg);        
        if (error & 0x1B) != 0x00
        {
            println!("--> ChipError, {}", error & 0x1B);
            return Err(TxpError::ChipError(error & 0x1B))
        }
        return Ok(());
    }

    fn write_data_to_fifo(&self, data: &[u8])
    {
        for d in data
        {
            // ToDo: Check if we can use a single transaction
            //       here
            self.write_byte(ChipRegisters::FIFODataReg as u8, *d)
        }
    }

    fn wait_irq(&self, irq_mask: u8, timeout_ms: u32) -> bool
    {
        for _ in 0..5
        {
            // This is a bit rubbish, however: The lowAlert IRQ
            // can apparently not be disabled, which means we will
            // get multiple IRQs, one that actually signalizes, the
            // event we're waiting for.
            if !self.tx_rdy_irq.wait_timeout(timeout_ms)
            {
                return false;
            }

            let irq = self.read_register(ChipRegisters::CommIrqReg);
            if irq & irq_mask as u8 != 0
            {
                return true;
            }
        }
        return false;
    }

    fn send_picc_command(&self, data: &[u8]) -> Result<Vec<u8>, TxpError>
    {
        self.enable_interrupt(IrqSources::RxIEn as u8);       
        self.clear_fifo();
        self.do_command(ChipCommand::IDLE);

        self.write_data_to_fifo(data);
        self.clear_irq_bits();     
        self.do_command(ChipCommand::TRANSCEIVE);

        // ToDo: Should we do this before actually triggering the TRANSCEIVE command?
        self.set_bit(ChipRegisters::BitFramingReg as u8, 0x80);

        if !self.wait_irq(IrqSources::RxIEn as u8, 75)
        {
            return Err(TxpError::Timeout);
        }

        self.clear_bit(ChipRegisters::BitFramingReg as u8, 0x80);

        // If we're here, we saw the correct IRQ and can now check,
        // if the command we triggered was successful by reading the
        // error register:
        let _ = self.handle_error()?;

        // let last_bits = self.read_register(ChipRegisters::ControlReg) & 0x07;
        // let back_len: u8;
        // if last_bits != 0
        // {
        //     back_len = (num_bytes_received - 1) * 8 + last_bits
        // }
        // else
        // {
        //     back_len = num_bytes_received * 8;
        // }
       
        return Ok(self.retrieve_fifo());
    }

    fn retrieve_fifo(&self) -> Vec<u8>
    {
        let num_bytes_received = self.read_register(ChipRegisters::FIFOLevelReg);
        let mut ret_val = Vec::<u8>::new();
        for _ in 0..num_bytes_received
        {
            ret_val.push(self.read_register(ChipRegisters::FIFODataReg));
        }
        ret_val
    }

    fn txp_request(&self, cmd: Iso1443aCommand) -> Result<Vec<u8>, TxpError>
    {
        self.write_mfrc522(ChipRegisters::BitFramingReg as u8, &[0x07 as u8]);
        self.send_picc_command(&[cmd as u8])
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
        self.write_mfrc522(ChipRegisters::BitFramingReg as u8, &[0x00 as u8]);
        // The 0x20 is actually the NVB!
        let mut res = self.send_picc_command( &[Iso1443aCommand::AnticollCasc1 as u8, 0x20])?;

        // The anti collision loop should go here... but alas:
        // Note, that we do not really support anti coll here, but we use the anticoll
        // procedure as specified for ISO14443A Tags to obtain the fullsize UID.
        if res.len() != 5
        {
            println!("--> Unsupported TagType");
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
        
        let mut  select_data = vec![Iso1443aCommand::AnticollCasc1 as u8, 0x70];
        select_data.append(&mut res);
        let sak = self.send_picc_command(&select_data)?[0];
        if sak != 0xB3
        {
            // TBD!
        }

        let uid2 = self.send_picc_command( &[Iso1443aCommand::AnticollCasc2 as u8, 0x20])?;
        let _ = self.check_bcc(&uid2)?;

        // check if this UID is complete (see 8371.TypeA_UID_retrieval.pdf)
        if uid2[0] != 0x88
        {
            res.extend_from_slice(&uid2[0..3]);
            return Ok(Vec::from_iter(res[0..7].iter().cloned()))
        }

        // ToDo: Which of these bytes are actually valid at this point
        res.extend_from_slice(&uid2[0..3]);

        // 10 Byte UID
        let uid3 = self.send_picc_command( &[Iso1443aCommand::AnticollCasc3 as u8, 0x20])?;
        let _ = self.check_bcc(&uid3)?;
        res.extend_from_slice(&uid3[0..3]);
        

        return Ok(Vec::from_iter(res[0..9].iter().cloned()))
    }

    pub fn search_txp(&self) -> Result<Vec<u8>, TxpError>
    {
        let atqa = self.txp_request(Iso1443aCommand::ReqA)?;
        // We should have received
        // an ATQA response containing at least the UID size of a txp, if
        // present.
        if atqa.len() != 2
        {
            println!("--> BadAtqa");
            return Err(TxpError::GeneralError);
        }


        let uid = self.txp_anticoll()?;
        Ok(uid)
    }
}
