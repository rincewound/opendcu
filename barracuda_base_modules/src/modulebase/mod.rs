use barracuda_core::{core::{SystemMessage, bootstage_helper::{boot, plain_boot}, broadcast_channel::{GenericReceiver, GenericSender}, channel_manager::ChannelManager}, trace::trace_helper};

use crate::cfg::ConfigMessage;

pub struct ModuleBase
{
    module_id               : u32,
    tracer                  : trace_helper::TraceHelper,
    pub cfg_rx              : GenericReceiver<ConfigMessage>,
    pub system_events_rx    : GenericReceiver<SystemMessage>,
    pub system_events_tx    : GenericSender<SystemMessage>,
}

impl ModuleBase
{
    pub fn new(module_id: u32, trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            module_id,
            tracer              : trace,
            cfg_rx              : chm.get_receiver(), 
            system_events_rx    : chm.get_receiver(),
            system_events_tx    : chm.get_sender(),
        }
    }

    pub fn boot<LliCb, HliCb>(&self,  llicb: Option<LliCb>, hlicb: Option<HliCb>)
    where LliCb: FnOnce() -> (), HliCb: FnOnce() -> ()
    {
        boot(self.module_id, llicb, hlicb, 
            &self.system_events_tx, 
            &self.system_events_rx, 
            &self.tracer);        
    }

    pub fn plain_boot(&self)
    {
        plain_boot(self.module_id, &self.system_events_tx, &self.system_events_rx, &self.tracer)
    }
}