
use crate::Trace::trace_message;

pub struct TraceHelper
{
    source_mod: String,
    trace_tx: crate::core::BroadcastChannel::GenericSender<crate::Trace::trace_message>
}

impl TraceHelper
{
    pub fn new(module_name: String, channelManager: &mut crate::core::ChannelManager::ChannelManager) -> Self
    {
        TraceHelper
        {
            source_mod: module_name,
            trace_tx: channelManager.get_sender::<crate::Trace::trace_message>()
        }
    }

    pub fn Trace(&self, message: String)
    {
        let finalMessage = format!("{}: {}", self.source_mod, message);        
        self.trace_tx.send(trace_message::new(finalMessage));
    }

    pub fn TraceStr(&self, message: &str)
    {
        let msgstring = String::from(message);
        self.Trace(msgstring);
    }
}