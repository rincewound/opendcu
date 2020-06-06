/*
*   # The MRFC522 Module
*   This module implements reading RFID media (ISO 14443A),
*   using the NXP MRFC522 reader chip
*
*   The code in this module is loosely based on 
*   "Pi My Life Up's guide on setting up an RFID RC522"
*   implementation
*
*   ## Usage
*   
*   ### Configuration
*   The module expects to be provided with a completely configured
*   SPI instance.
*   A this point it only supports reading the UID of ISO 14443A tags.
*
*
*   ### Behavior
*   After the module is started it will search for media every 50 ms
*   and generate door-open requests everytime it sees a medium.
*
*   ### Notes
*   The original code does not use IRQs, but instead uses polling
*   (stupidly!). This should be 
*/

use crate::{core::
            {bootstage_helper::{boot_noop, boot}, 
             channel_manager::ChannelManager, 
             broadcast_channel::{GenericSender, GenericReceiver}, SystemMessage}, 
             lowlevel::{spi::SpiInterface, interrupt::Interrupt}, 
             trace::trace_helper, 
             modcaps::{ModuleCapability, ModuleCapabilityAdvertisement}, acm::WhitelistAccessRequest
            };
use std::{thread, sync::Arc};

mod mfrc522;

const MODULE_ID: u32 = 0x0B000000;

pub fn launch<Spi, Irq>(chm: &mut ChannelManager, spi_driver: Spi, tx_ready_irq: Irq)
    where Spi: SpiInterface+Send + 'static, Irq: Interrupt+Send+ 'static
{    
    let tracer = trace_helper::TraceHelper::new("ARM/MFRC522".to_string(), chm);
    let mut rm = ReaderModule::new(tracer, chm, spi_driver, tx_ready_irq);
    thread::spawn(move || {  
        rm.init();   
        loop 
        {
            rm.search_media();
        }   
        
    });
}

pub struct ReaderModule<Spi, Irq>
    where Spi: SpiInterface, Irq: Interrupt
{            
    system_events_rx: Arc<GenericReceiver<SystemMessage>>,
    system_events_tx: GenericSender<SystemMessage>,
    modcaps_tx:  GenericSender<ModuleCapabilityAdvertisement>,
    access_request_tx: GenericSender<crate::acm::WhitelistAccessRequest>,
    tracer: trace_helper::TraceHelper,
    last_txp: Option<Vec<u8>>,
    rfchip: mfrc522::mfrc522<Spi,Irq>
}

impl<Spi: SpiInterface, Irq: Interrupt> ReaderModule<Spi, Irq> 
{
    pub fn new(tracer: trace_helper::TraceHelper, chm: &mut ChannelManager, spi_driver: Spi, tx_rdy_irq: Irq) -> Self
    {
        Self
        {   
            system_events_rx: chm.get_receiver(),
            system_events_tx: chm.get_sender(),
            modcaps_tx: chm.get_sender(),
            access_request_tx: chm.get_sender(),
            tracer,
            rfchip: mfrc522::mfrc522::new(spi_driver, tx_rdy_irq),
            last_txp: None
        }
    }

    pub fn init(&self)
    {
        let modcaps_tx_clone =self.modcaps_tx.clone();
        let hlicb= Some(move|| {
            let m = ModuleCapabilityAdvertisement {
                caps: vec![ModuleCapability::AccessPoints(1)],
                module_id: MODULE_ID
            };
            modcaps_tx_clone.send(m);            
        });

        boot(MODULE_ID, Some(boot_noop), hlicb, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);
    }

    fn is_new_txp(&self, uid: &Vec<u8>) -> bool
    {
        match self.last_txp
        {            
            Some(ref last_uid) => {
                let zip_iter = last_uid.iter().zip(uid.iter());
                for (byte_a, byte_b) in zip_iter
                {   
                    if byte_a != byte_b
                    {
                        return true;
                    }
                }
            },
            None => return true
        }
        return false;
    }

    pub fn search_media(&mut self)
    {
        self.rfchip.toggle_antenna(true);
        let txp = self.rfchip.search_txp();
        self.rfchip.toggle_antenna(false);
        if let Ok(uid) = txp
        {
            // found a txp, check if we have seen this one before:
            if !self.is_new_txp(&uid)
            {
                return;
            }
            self.tracer.trace_str("Found new transponder.");

            let req = WhitelistAccessRequest
            {
                access_point_id: MODULE_ID & 0x01,
                identity_token_number: uid.clone()
            };

            self.access_request_tx.send(req);
            self.last_txp = Some(uid);
        }
    }
}