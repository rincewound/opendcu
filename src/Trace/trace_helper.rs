
use crate::trace::TraceMessage;

pub struct TraceHelper
{
    source_mod: String,
    trace_tx: crate::core::broadcast_channel::GenericSender<TraceMessage>
}

impl TraceHelper
{
    pub fn new(module_name: String, channel_manager: &mut crate::core::channel_manager::ChannelManager) -> Self
    {
        TraceHelper
        {
            source_mod: module_name,
            trace_tx: channel_manager.get_sender::<TraceMessage>()
        }
    }

    pub fn Trace(&self, message: String)
    {
        let finalMessage = format!("{}: {}", self.source_mod, message);        
        self.trace_tx.send(TraceMessage::new(finalMessage));
    }

    pub fn TraceStr(&self, message: &str)
    {
        let msgstring = String::from(message);
        self.Trace(msgstring);
    }
}