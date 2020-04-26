
use crate::core::channel_manager::*;
use crate::core::event::DataEvent;
use std::{io, thread, self};
use std::{sync::Arc, io::Write};

use crate::core::SystemMessage;


pub mod trace_helper;

const MODULE_ID: u32 = 0x02000000;

#[derive(Clone)]
pub struct TraceMessage
{
    msg: String
}

impl TraceMessage
{
    pub fn new(msg: String) -> Self
    {
        TraceMessage {
            msg: msg
        }
    }

    pub fn from_str(msg: &str) -> Self
    {
        TraceMessage {
            msg: msg.to_string()
        }
    }
}

pub fn launch(chm: &mut ChannelManager)
{    
    let trace_rx = chm.get_receiver::<TraceMessage>();
    let sys_rx= chm.get_receiver::<crate::core::SystemMessage>();
    let sys_tx= chm.get_sender::<crate::core::SystemMessage>();
    sys_tx.send(SystemMessage::StageComplete(crate::core::BootStage::Sync, MODULE_ID));
    println!("Trace active");

    let _ = thread::Builder::new().name("Trace".to_string()).spawn(move || {
        loop
        {
            let queue = select_chan!(sys_rx, trace_rx);
            if queue == 1
            {
                let message = trace_rx.receive();
                println!("{}", message.msg);
                let _ = io::stdout().flush();
            }
            if queue == 0
            {
                let msg = sys_rx.receive();
                if let SystemMessage::RunStage(x) = msg
                {
                    
                    println!("Ran bootstage {}", x as u32);
                    sys_tx.send(SystemMessage::StageComplete(x, MODULE_ID));
                }
            }        
        }
    });
}