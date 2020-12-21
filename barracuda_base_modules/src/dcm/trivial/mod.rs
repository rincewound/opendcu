use barracuda_core::{core::{SystemMessage, bootstage_helper::{self}, broadcast_channel::{GenericReceiver, GenericSender}, channel_manager::ChannelManager}, trace::trace_helper};

use std::thread;

use crate::io;

const MODULE_ID: u32 = 0x08000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("DCM/Trivial".to_string(), chm);
    let tdc = TrivialDoorControl::new(tracer, chm);
    thread::spawn(move || {  
        tdc.init();   
        loop 
        {
            if !tdc.do_request()
            {
                break;
            }
        }   
        
    });
}

/// # Trivial Door Control Module
/// This is the most basic door control module possible.
/// It will literally just control a single output (i.e. an
/// electric door opener/buzzer) when confronted with
/// a door open request. Also this output happens to be out0, 
/// always.
pub struct TrivialDoorControl
{
    tracer: trace_helper::TraceHelper,
    system_events_rx: GenericReceiver<SystemMessage>,
    system_events_tx: GenericSender<SystemMessage>,
    door_requests_rx: GenericReceiver<crate::dcm::DoorOpenRequest>,
    output_cmd_tx: GenericSender<io::OutputSwitch>
}

impl TrivialDoorControl
{
    pub fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        TrivialDoorControl
        {            
            tracer              : trace,
            system_events_rx    : chm.get_receiver(),
            system_events_tx    : chm.get_sender(),
            door_requests_rx    : chm.get_receiver(),
            output_cmd_tx       : chm.get_sender()
        }
    }

    pub fn init(&self)
    {
        bootstage_helper::plain_boot(MODULE_ID, &self.system_events_tx, &self.system_events_rx, &self.tracer)
    }

    pub fn do_request(&self) -> bool
    {
        let request = self.door_requests_rx.receive();

        self.tracer.trace(format!("Open door {}", request.access_point_id));

        // ToDo: 
        // * Use switchtime configuration
        let cmd = io::OutputSwitch{output_id: request.access_point_id, target_state: io::OutputState::High, switch_time: 5000};
        self.output_cmd_tx.send(cmd);

        return true;
    }
}