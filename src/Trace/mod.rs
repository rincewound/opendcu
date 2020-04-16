
use crate::core::ChannelManager::*;
#[macro_use]
use crate::core;
use crate::core::Event::DataEvent;
use std::{io, thread, self};
use std::{sync::Arc, io::Write};

use crate::core::SystemMessage;


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
    let trace_rx = chm.get_receiver::<trace_message>();
    let sys_rx= chm.get_receiver::<crate::core::SystemMessage>();
    let sys_tx= chm.get_sender::<crate::core::SystemMessage>();
    sys_tx.send(SystemMessage::StageComplete(crate::core::BootStage::Sync));
    println!("Trace active");

    let thrd = thread::Builder::new().name("Trace".to_string()).spawn(move || {
        loop
        {
            let queue = select_chan!(sys_rx, trace_rx);
            if queue == 1
            {
                let message = trace_rx.receive();
                println!("{}", message.msg);
                io::stdout().flush();
            }
            if queue == 0
            {
                let msg = sys_rx.receive();
                if let SystemMessage::RunStage(x) = msg
                {
                    
                    println!("Ran bootstage {}", x as u32);
                    sys_tx.send(SystemMessage::StageComplete(x));
                }
            }        
        }
    });
}