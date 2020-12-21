use crate::{cfg::ConfigMessage, trace::trace_helper};
use super::{SystemMessage, broadcast_channel::{GenericSender, GenericReceiver}, channel_manager::ChannelManager, bootstage_helper::boot};
use std::sync::Arc;

pub struct ModuleBase
{
    module_id               : u32,
    tracer                  : trace_helper::TraceHelper,
    pub cfg_rx              : Arc<GenericReceiver<ConfigMessage>>,
    pub system_events_rx    : Arc<GenericReceiver<SystemMessage>>,
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
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);        
    }

    pub fn plain_boot(&self)
    {
        crate::core::bootstage_helper::plain_boot(self.module_id, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer)
    }
}