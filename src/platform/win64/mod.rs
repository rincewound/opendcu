use crate::{trace::trace_helper, core::{broadcast_channel::{GenericReceiver, GenericSender}, channel_manager::ChannelManager, bootstage_helper::boot}, modcaps::{ModuleCapability, ModuleCapabilityAdvertisement}};
use std::{thread, sync::Arc};

const MODULE_ID: u32 = 0x09000000;

pub fn launch(chm: &mut ChannelManager)   
{    
    let tracer = trace_helper::TraceHelper::new("Plattform/Win32Io".to_string(), chm);
    let ioman = w32Io::new(tracer, chm);
    thread::spawn(move || {  
        ioman.init();           
    });
}

struct w32Io
{
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    modcaps_tx:  GenericSender<crate::modcaps::ModuleCapabilityAdvertisement>,
    tracer: trace_helper::TraceHelper
}


impl w32Io
{
    pub fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        w32Io {        
         system_events_rx: chm.get_receiver(),
         system_events_tx: chm.get_sender(),
         modcaps_tx: chm.get_sender(),
         tracer: trace
        }
    }

    pub fn init(&self)
    {
        let modcaps_tx_clone =self.modcaps_tx.clone();
        let cbs= [Some(move|| {
            /*
                This is executed during LLI
            */
            let m = ModuleCapabilityAdvertisement {
                caps: vec![ModuleCapability::Outputs(3), ModuleCapability::Inputs(3)],
                module_id: MODULE_ID
            };
            modcaps_tx_clone.send(m);            
        }), None];

        boot(MODULE_ID, cbs, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);
    }
}