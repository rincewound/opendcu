
use crate::core::broadcast_channel::*;
use crate::core::channel_manager::*;
use crate::trace::*;
use crate::acm::*;
use std::{sync::Arc, thread};
use std::io;


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
}

impl ConsoleInput
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        ConsoleInput
        {
            tracer: trace,
            access_request_tx: chm.get_sender::<crate::acm::WhitelistAccessRequest>(),
            system_events_rx: chm.get_receiver::<crate::core::SystemMessage>(),
            system_events_tx: chm.get_sender::<crate::core::SystemMessage>(),
        }
    }

    pub fn init(&mut self)
    {
        crate::core::bootstage_helper::plain_boot(MODULE_ID, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer);        
    }

    pub fn do_request(&mut self) -> bool
    {

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let req = WhitelistAccessRequest
                {
                    access_point_id: MODULE_ID & 0x01,      // Access point 1 
                    identity_token_number: input.into_bytes()
                };
                self.access_request_tx.send(req);
            }
            Err(error) => println!("error: {}", error),
        }
        true
    }
}