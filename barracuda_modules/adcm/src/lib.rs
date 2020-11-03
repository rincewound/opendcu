
use barracuda_core::core::broadcast_channel::*;
use barracuda_core::core::channel_manager::*;
use barracuda_core::core::{bootstage_helper::*, event::DataEvent};
use barracuda_core::{Handler, cfg::{cfgholder::*, self}};
use barracuda_core::trace::*;
use barracuda_core::{io::InputEvent, dcm::DoorOpenRequest, profile::ProfileChangeEvent, select_chan, wait_for};
use std::{sync::Arc, thread};
use std::fs::File;
use components::Passageway;

use crate::components::serialization_types::{PassagewaySetting};


mod components;

const MODULE_ID: u32 = 0x0D000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("DCM/ADCM".to_string(), chm);
    let mut adcm = ADCM::new(tracer, chm);
    adcm.init(chm);   
    thread::spawn(move || {          
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
    module_base         : barracuda_core::core::module_base::ModuleBase,
    bin_prof_rx         : Arc<GenericReceiver<ProfileChangeEvent>>,  
    input_rx            : Arc<GenericReceiver<InputEvent>>, 
    door_req_rx         : Arc<GenericReceiver<DoorOpenRequest>>,
    passageways         : Vec<Passageway>
}

impl ADCM
{
    pub fn new(tracer: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            module_base         : barracuda_core::core::module_base::ModuleBase::new(MODULE_ID, tracer, chm),
            bin_prof_rx         : chm.get_receiver(),
            input_rx            : chm.get_receiver(),
            door_req_rx         : chm.get_receiver(),
            passageways         : vec![]
        }
    }

    pub fn init(&mut self, chm: &mut ChannelManager)
    {
        // Load passageways
        // ToDo: We might be better off here by just using Util::ObjectStore
        let reader = File::open("./passageways.txt");
        if let Ok(file) = reader
        {
            let passageway_settings: Vec<PassagewaySetting> = serde_json::from_reader(file).unwrap_or_else(|_| Vec::new());

            for setting in passageway_settings.into_iter()
            {
                self.passageways.push(Passageway::new(setting, chm));
            }
        }

        let the_receiver = self.module_base.cfg_rx.clone(); 
        let hli_cb = Some(|| {

            let res = the_receiver.receive();
            let cfg::ConfigMessage::RegisterHandlers(cfg_holder) = res;
            let mut holder = cfg_holder.lock();

            holder.register_handler(FunctionType::Put, "passageway".to_string(), Handler!(|pway: PassagewaySetting|
                {
                    Self::process_passageway_setting(pway)
                }));

            holder.register_handler(FunctionType::Delete, "passageway".to_string(), Handler!(|pway: PassagewaySetting|
                {
                    Self::process_delete_passageway(pway);
                }));            
        });

        self.module_base.boot(Some(boot_noop), hli_cb);
    }

    fn process_passageway_setting(passageway: PassagewaySetting)
    {
        
    }

    fn process_delete_passageway(passageway: PassagewaySetting)
    {

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