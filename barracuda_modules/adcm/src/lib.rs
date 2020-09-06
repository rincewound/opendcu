
use barracuda_core::core::broadcast_channel::*;
use barracuda_core::core::channel_manager::*;
use barracuda_core::core::{shareable::Shareable, bootstage_helper::*, SystemMessage, event::DataEvent};
use barracuda_core::{Handler, cfg::{ConfigMessage, cfgholder::*, self}};
use barracuda_core::trace::*;
use barracuda_core::{io::InputEvent, dcm::DoorOpenRequest, profile::ProfileChangeEvent, select_chan, wait_for};
use std::{sync::Arc, thread};
use components::Passageway;

mod components;

const MODULE_ID: u32 = 0x03000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("DCM/ADCM".to_string(), chm);
    let mut adcm = ADCM::new(tracer, chm);
    thread::spawn(move || {  
        adcm.init();   
        loop 
        {
            if !adcm.run()
            {
                break;
            }
        }   
        
    });
}

struct ADCM
{
    bin_prof_rx: Arc<GenericReceiver<ProfileChangeEvent>>,  
    input_rx: Arc<GenericReceiver<InputEvent>>, 
    door_req_rx: Arc<GenericReceiver<DoorOpenRequest>>,
    passageways: Vec<Passageway>
}

impl ADCM
{
    pub fn new(tracer: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            bin_prof_rx: chm.get_receiver(),
            input_rx: chm.get_receiver(),
            door_req_rx: chm.get_receiver(),
            passageways: vec![]
        }
    }

    pub fn init(&mut self)
    {
        // Load passageways!
    }

    pub fn run(&mut self) -> bool
    {        
        let queue_id = select_chan!(self.bin_prof_rx, self.input_rx, self.door_req_rx);
        match queue_id
        {
            0 => self.do_bin_prof_event(),
            1 => self.do_input_event(),
            2 => self.do_door_request(),
            _ => return false
        }
        true
    }

    fn do_input_event(&mut self)
    {
        let input_event = self.input_rx.receive();
        for passageway in self.passageways.iter_mut()
        {            
            passageway.on_input_change(&input_event);
        }
    }

    fn do_bin_prof_event(&mut self)
    {
        let binprof_event = self.bin_prof_rx.receive();
        for passageway in self.passageways.iter_mut()
        {            
            passageway.on_profile_change(&binprof_event);
        }
    }

    fn do_door_request(&mut self)
    {
        let door_request = self.door_req_rx.receive();
        for passageway in self.passageways.iter_mut()
        {            
            passageway.on_door_open_request(&door_request);
        }
    }

}