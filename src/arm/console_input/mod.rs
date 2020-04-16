

use crate::core::BootStage;
use crate::core::BroadcastChannel::*;
use crate::core::{SystemMessage, ChannelManager::*};
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

    fn wait_for_stage(&self, stage: BootStage)
    {
        self.tracer.Trace(format!("Wait for stage signal {}", stage as u32));
        loop
        {
            let msg = self.system_events_rx.receive();
            match msg
            {
                SystemMessage::RunStage(s) => if s == stage {
                    break;
                },
                _ => continue /*ABORTS!*/
            }
        }  
    }

    fn send_stage_complete(&self, stage: BootStage)
    {
        self.system_events_tx.send(crate::core::SystemMessage::StageComplete(stage, Module_ID));
    }

    pub fn init(&mut self)
    {
        self.tracer.TraceStr("Starting");
        self.send_stage_complete(BootStage::Sync);

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