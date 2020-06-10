use barracuda_core::lowlevel::spi::{SpiInterface};
use barracuda_core::lowlevel::interrupt::Interrupt;

use barracuda_core::core::event::Event;

use rppal::spi::*;
use rppal::gpio::*;
use std::sync::Arc;


pub struct RfidSpi
{
    spi: Spi
}


impl RfidSpi
{
    pub fn new() -> Self
    {
        // ToDo: Check correct spi settings for MFRC522 on RasPi/Reference, most notably spi mode!
        let spi_interface = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1000000, rppal::spi::Mode::Mode0).unwrap();

        Self
        {
            spi: spi_interface
        }
    }    
}

impl SpiInterface for RfidSpi
{
    fn send_receive(&self, data: &[u8]) -> Vec<u8> {
        let mut receive_buf = Vec::from(data);
        let _ = self.spi.transfer(&mut receive_buf.as_mut_slice(), data);
        return receive_buf;
    }    
}

pub struct RfidIrq
{
    irq_event: Arc<Event>,
    _pin: InputPin       // must stay in scope for the IRQs to stay active!
}

impl RfidIrq
{
    pub fn new() -> Self
    {
        let gpio = Gpio::new().unwrap();
        let mut pin = gpio.get(23).unwrap().into_input_pullup();       // ToDo: Check appropiate pin!
        let event = Arc::new(Event::new());
        let evt_clone = event.clone();
        let _ = pin.set_async_interrupt(Trigger::FallingEdge, move |_arg| {
            println!("TXRdy IRQ seen");
            evt_clone.trigger()
        });

        RfidIrq
        {
            irq_event: event.clone(),
            _pin: pin
        }
    }
}

impl Interrupt for RfidIrq
{
    fn wait(&self) {
        self.irq_event.wait();
    }

    fn wait_timeout(&self,timeout_ms: u32) -> bool {
        self.irq_event.wait_with_timeout(timeout_ms as u64)
    }    
}
