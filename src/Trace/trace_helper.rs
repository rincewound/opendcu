
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

    pub fn trace(&self, message: String)
    {
        let final_message = format!("{}: {}", self.source_mod, message);        
        self.trace_tx.send(TraceMessage::new(final_message));
    }

    pub fn trace_str(&self, message: &str)
    {
        let msgstring = String::from(message);
        self.trace(msgstring);
    }
}