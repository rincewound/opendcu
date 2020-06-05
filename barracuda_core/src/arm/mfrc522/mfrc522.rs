use super::spi::SpiInterface;
use super::interrupt::Interrupt;

enum ChipCommand
{
    IDLE             = 0x00,
    MEM              = 0x01,
    GenerateRandomId = 0x02,
    TRANSMIT         = 0x04,
    NoCmdChange      = 0x07,
    CALCCRC          = 0x03,    
    RECEIVE          = 0x08,    
    TRANSCEIVE       = 0x0C,
    AUTHENT          = 0x0E,
    RESETPHASE       = 0x0F,
    
}

enum PicCommand
{
    PICC_REQIDL = 0x26,
    PICC_REQALL = 0x52,
    PICC_ANTICOLL = 0x93,
    //PICC_SElECTTAG = PICC_ANTICOLL,
    PICC_AUTHENT1A = 0x60,
    PICC_AUTHENT1B = 0x61,
    PICC_READ = 0x30,
    PICC_WRITE = 0xA0,
    PICC_DECREMENT = 0xC0,
    PICC_INCREMENT = 0xC1,
    PICC_RESTORE = 0xC2,
    PICC_TRANSFER = 0xB0,
    PICC_HALT = 0x50 
}

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

pub struct mfrc522<T, Irq>
    where T: SpiInterface, Irq: super::interrupt::Interrupt
{
    spi_interface: T,
    tx_rdy_irq: Irq
}

impl<T: SpiInterface, Irq: Interrupt> mfrc522<T, Irq>
{

    pub fn open(&self)
    {
        // ToDo: Should we configure the SPI device here, or
        // do we expect to get it readily configured for the
        // target?
        //self.spi_interface
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

    fn toggle_antenna(&self, enable_antenna: bool)
    {
        let tx_ctrl = self.read_register(ChipRegisters::TxControlReg);
        if enable_antenna
        {
            if (tx_ctrl & 0x03) != 0x03
            {
                self.set_bit(ChipRegisters::TxControlReg as u8, 0x03);
            }
        }
        else
        {
            self.clear_bit(ChipRegisters::TxControlReg as u8, 0x03);
        }
    }

    fn send_txp_command(&self, command: ChipCommand, data: &[u8])
    {

        // Step 1: Setup IRQs to wait for
        // self.Write_MFRC522(self.CommIEnReg, irqEn | 0x80)
        // self.ClearBitMask(self.CommIrqReg, 0x80)
        // self.SetBitMask(self.FIFOLevelReg, 0x80)

        self.do_command(ChipCommand::IDLE);
        for d in data
        {
            // ToDo: Check if we can use a single transaction
            //       here
            self.write_byte(ChipRegisters::FIFODataReg as u8, *d)
        }

        self.do_command(command);

        // Stupid: let's use an IRQ instead...
        // i = 2000
        // while True:
        //     n = self.Read_MFRC522(self.CommIrqReg)
        //     i -= 1
        //     if ~((i != 0) and ~(n & 0x01) and ~(n & waitIRq)):
        //         break
        if !self.tx_rdy_irq.wait_timeout(2000)
        {
            // Timeout!
            return;
        }

        self.clear_bit(ChipRegisters::BitFramingReg as u8, 0x80);

        // Read error register

    }
}