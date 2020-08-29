
use crate::core::broadcast_channel::*;
use crate::core::channel_manager::*;
use crate::trace::*;
use crate::acm::*;
use std::{sync::Arc, thread};
use std::io;

use crate::{core::
    {bootstage_helper::{boot_noop, boot}, 
     channel_manager::ChannelManager, 
     broadcast_channel::{GenericSender, GenericReceiver}, SystemMessage},              
     trace::trace_helper, 
     modcaps::{ModuleCapability, ModuleCapabilityAdvertisement}, acm::WhitelistAccessRequest
    };

const MODULE_ID: u32 = 0x04000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("ARM/ConsoleInput".to_string(), chm);
    let mut wl = ConsoleInput::new(tracer, chm);
    thread::spawn(move || {  
        wl.init();   
        loop 
        {
            if !wl.do_request()
            {
                break;
            }
        }   
        
    });
}

struct ConsoleInput
{
    tracer: trace_helper::TraceHelper,
    access_request_tx: GenericSender<crate::acm::WhitelistAccessRequest>,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    modcaps_tx:  GenericSender<ModuleCapabilityAdvertisement>,
}

impl ConsoleInput
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        ConsoleInput
        {
            tracer: trace,
            access_request_tx: chm.get_sender(),
            system_events_rx: chm.get_receiver(),
            system_events_tx: chm.get_sender(),
            modcaps_tx: chm.get_sender(),
        }
    }

    pub fn init(&mut self)
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

        // crate::core::bootstage_helper::plain_boot(MODULE_ID, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer);        
    }

    pub fn do_request(&mut self) -> bool
    {

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let req = WhitelistAccessRequest
                {
                    access_point_id: MODULE_ID | 0x01,      // Access point 1 
                    identity_token_number: input.into_bytes()
                };
                self.access_request_tx.send(req);
            }
            Err(error) => println!("error: {}", error),
        }
        true
    }
}