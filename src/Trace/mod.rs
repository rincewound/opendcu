
use crate::core::ChannelManager::*;
use std::{io, thread, self};
use std::io::Write;

pub mod TraceHelper;

#[derive(Clone)]
pub struct trace_message
{
    msg: String
}

impl trace_message
{
    pub fn new(msg: String) -> Self
    {
        trace_message {
            msg: msg
        }
    }

    pub fn from_str(msg: &str) -> Self
    {
        trace_message {
            msg: msg.to_string()
        }
    }
}

pub fn launch(chm: &mut ChannelManager)
{
    chm.register_channel::<trace_message>();
    let trace_rx = chm.get_receiver::<trace_message>().unwrap();
    thread::spawn(move || {
        loop
        {
            let message = trace_rx.receive();
            println!("{}", message.msg);
            io::stdout().flush();
        }
    });
}