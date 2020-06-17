
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

// Missing DivIrqReg 

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

pub enum StatusRegister2Bits
{
    TempSensClear   = 0x80,
    I2CForcesHS     = 0x40,
    RFU             = 0x30, // Bits 4 and 5
    MFCrypto1On     = 0x08,
    ModemState      = 0x07  // bits 0 to 2
}

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
    REQA                = 0x26,     // AKA REQIDL
    REQALL              = 0x52,
    ANTICOLL_CASC1      = 0x93,     // is also select_tag in original.
    ANTICOLL_CASC2      = 0x95,
    ANTICOLL_CASC3      = 0x97,
    AUTHENT1A           = 0x60,
    AUTHENT1B           = 0x61,
    READ                = 0x30,
    WRITE               = 0xA0,
    DECREMENT           = 0xC0,
    INCREMENT           = 0xC1,
    RESTORE             = 0xC2,
    TRANSFER            = 0xB0,
    HALT                = 0x50 
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

pub struct mfrc522<T, Irq>
    where T: SpiInterface, Irq: Interrupt
{
    spi_interface: T,
    tx_rdy_irq: Irq
}

impl<T, Irq> mfrc522<T, Irq>
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
        result.clear_bit(ChipRegisters::CommIEnReg as u8, IrqSources::HiAlertIEn as u8);
        result.clear_bit(ChipRegisters::CommIEnReg as u8, IrqSources::LoAlertIEn as u8);

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

    pub fn Reset(&self)
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

    fn send_chip_command(&self, command: ChipCommand, data: &[u8]) -> Result<Vec<u8>, TxpError>
    {

        // Step 1: Setup IRQs to wait for
        let mut irq_en: u8 = 0x00;
        let mut irq_id: u8 = 0x00;

        if command == ChipCommand::AUTHENT
        {
            irq_en = 0x12;
            irq_id = 0x10;
        }

        if command == ChipCommand::TRANSCEIVE
        {
            irq_en = 0x77;
            irq_id = 0x30;
        }

        self.write_mfrc522(ChipRegisters::CommIEnReg as u8, &[irq_en | 0x80]);
        self.clear_bit(ChipRegisters::CommIrqReg as u8, 0x80);
        self.clear_bit(ChipRegisters::FIFOLevelReg as u8, 0x80);

        self.do_command(ChipCommand::IDLE);
        for d in data
        {
            // ToDo: Check if we can use a single transaction
            //       here
            self.write_byte(ChipRegisters::FIFODataReg as u8, *d)
        }

        self.do_command(command);

        if command == ChipCommand::TRANSCEIVE
        {
            self.set_bit(ChipRegisters::BitFramingReg as u8, 0x80)
        }

        // Stupid: let's use an IRQ instead...
        let mut i = 3500;
        while true
        {
            let n = self.read_register(ChipRegisters::CommIrqReg);
            i -= 1;
            if (n & irq_id) != 0
            {
                //println!("IRQ seen : {}", n);
                break;
            }
            if i <= 0
            {
                break;
            }
        }

        // if !self.tx_rdy_irq.wait_timeout(75)
        // {
        //     // Timeout!
        //     println!("--> Timeout!");
        //     return Err(TxpError::Timeout)
        // }

        if i <= 0
        {
            return Err(TxpError::Timeout);         
        }

        // let irq_no = self.read_register(ChipRegisters::CommIrqReg);
        // if irq_no != irq_id
        // {
        //     // wrong IRQ triggered, abort.
        //     println!("--> GeneralError!");
        //     return Err(TxpError::GeneralError);
        // }

        self.clear_bit(ChipRegisters::BitFramingReg as u8, 0x80);

        // If we're here, we saw the correct IRQ and can now check,
        // if the command we triggered was successful by reading the
        // error register:
        let error = self.read_register(ChipRegisters::ErrorReg);        
        if (error & 0x1B) != 0x00
        {
            return Err(TxpError::ChipError(error & 0x1B))
        }

        // Check the number of bytes we received
        let num_bytes_received = self.read_register(ChipRegisters::FIFOLevelReg);
        let last_bits = self.read_register(ChipRegisters::ControlReg) & 0x07;

        let back_len: u8;
        if last_bits != 0
        {
            back_len = (num_bytes_received - 1) * 8 + last_bits
        }
        else
        {
            back_len = num_bytes_received * 8;
        }
       
        let mut ret_val = Vec::<u8>::new();
        for _ in 0..num_bytes_received
        {
            ret_val.push(self.read_register(ChipRegisters::FIFODataReg));
        }
        return Ok(ret_val);
    }

    pub fn txp_request(&self, cmd: Iso1443aCommand) -> Result<Vec<u8>, TxpError>
    {
        self.write_mfrc522(ChipRegisters::BitFramingReg as u8, &[0x07 as u8]);
        self.send_chip_command(ChipCommand::TRANSCEIVE, &[cmd as u8])
    }

    pub fn txp_anticoll(&self)-> Result<Vec<u8>, TxpError>
    {
        self.write_mfrc522(ChipRegisters::BitFramingReg as u8, &[0x00 as u8]);
        let res = self.send_chip_command(ChipCommand::TRANSCEIVE, &[Iso1443aCommand::ANTICOLL_CASC1 as u8, 0x20])?;

        // The anti collision loop should go here... but alas:
        if res.len() != 5
        {
            return Err(TxpError::UnsupportedTagType);
        }

        // The last byte of the data contains an XOR blockcheck.
        // this needs to be done in case the reception was interrupted
        // and we only got parts of the UID.
        let mut bcc: u8 = 0x00;
        for idx in 0..res.len() - 1
        {
            bcc = bcc ^ res[idx]
        }

        if bcc != res[res.len() - 1]
        {
            return Err(TxpError::CommunicationLost);
        }

        return Ok(Vec::from_iter(res[0..4].iter().cloned()))
    }

    pub fn search_txp(&self) -> Result<Vec<u8>, TxpError>
    {
        let atqa = self.txp_request(Iso1443aCommand::REQA)?;
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
}
