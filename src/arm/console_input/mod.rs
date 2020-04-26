
use crate::core::BroadcastChannel::*;
use crate::core::ChannelManager::*;
use crate::Trace;
use crate::acm::*;
use std::{sync::Arc, thread};
use std::io;


const Module_ID: u32 = 0x04000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = Trace::TraceHelper::TraceHelper::new("ARM/ConsoleInput".to_string(), chm);
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
    tracer: Trace::TraceHelper::TraceHelper,
    access_request_tx: GenericSender<crate::acm::WhitelistAccessRequest>,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
}

impl ConsoleInput
{
    fn new(trace: Trace::TraceHelper::TraceHelper, chm: &mut ChannelManager) -> Self
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
        crate::core::BootstageHelper::Boot(Module_ID, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer);        
    }

    pub fn do_request(&mut self) -> bool
    {

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                let req = WhitelistAccessRequest
                {
                    access_point_id: 0,
                    identity_token_number: input.into_bytes()
                };
                self.access_request_tx.send(req);
            }
            Err(error) => println!("error: {}", error),
        }
        true
    }
}