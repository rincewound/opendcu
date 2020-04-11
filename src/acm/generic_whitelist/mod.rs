

use crate::core::BootStage;
use crate::core::BroadcastChannel::*;
use crate::core::{SystemMessage, ChannelManager::*};
use crate::Trace;
use crate::acm::*;
use std::{sync::Arc, thread};

mod whitelist;

pub fn launch(chm: &ChannelManager)
{    
    let tracer = Trace::TraceHelper::TraceHelper::new("ACM/Whitelist".to_string(), chm);
    let mut wl = GenericWhitelist::new(tracer, chm, whitelist::SqliteEntryProvider);
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

struct GenericWhitelist<WhitelistProvider: whitelist::WhitelistEntryProvider>
{
    tracer: Trace::TraceHelper::TraceHelper,
    access_request_rx: Arc<GenericReceiver<crate::acm::WhitelistAccessRequest>>,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    whitelist: WhitelistProvider
}

impl<WhitelistProvider: whitelist::WhitelistEntryProvider> GenericWhitelist<WhitelistProvider>
{
    fn new(trace: Trace::TraceHelper::TraceHelper, chm: &ChannelManager, whitelist: WhitelistProvider) -> Self
    {
        GenericWhitelist
        {
            tracer: trace,
            access_request_rx: chm.get_receiver::<crate::acm::WhitelistAccessRequest>().unwrap(),
            system_events_rx: chm.get_receiver::<crate::core::SystemMessage>().unwrap(),
            system_events_tx: chm.get_sender::<crate::core::SystemMessage>().unwrap(),
            whitelist
        }
    }

    fn wait_for_stage(&self, stage: BootStage)
    {
        loop
        {
            let msg = self.system_events_rx.receive();
            match msg
            {
                SystemMessage::RunStage(s) => if s == stage {break;},
                _ => return /*ABORTS!*/
            }
        }  
    }

    fn send_stage_complete(&self, stage: BootStage)
    {
        self.system_events_tx.send(crate::core::SystemMessage::StageComplete(stage));
    }

    pub fn init(&mut self)
    {
        self.tracer.TraceStr("Starting");

        self.wait_for_stage(BootStage::LowLevelInit);
        self.tracer.TraceStr("Runstage: LLI");
        self.send_stage_complete(BootStage::LowLevelInit);

        self.wait_for_stage(BootStage::HighLevelInit);
        self.tracer.TraceStr("Runstage: HLI");
        self.send_stage_complete(BootStage::HighLevelInit);

        self.wait_for_stage(BootStage::Application);
        self.tracer.TraceStr("Runstage: APP");
    }

    pub fn do_request(&mut self) -> bool
    {
        let req = self.access_request_rx.receive();
        // ToDo: This should be done from a threadpool.
        self.process_access_request(req);
        true
    }

    fn process_access_request(&self, req: WhitelistAccessRequest)
    {
        // Pull Whitelist Entry
        let entry = self.whitelist.get_entry(req.identity_token_number);

        // Found? If so, check access profile, otherwise emit AccessDenied Sig
        if let Some(entry) = entry {
            
            // Good? If so, emit DoorOpenRequest, otherwise emit AccessDenied Sig          
        }
        else
        {
            // Emit Access Denied
        }
    }
}