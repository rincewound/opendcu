
use barracuda_core::core::{shareable::Shareable, broadcast_channel::*};
use barracuda_core::core::channel_manager::*;
use barracuda_core::core::{bootstage_helper::*, event::DataEvent};
use barracuda_core::{Handler, cfg::{cfgholder::*, self}};
use barracuda_core::trace::*;
use barracuda_core::{io::InputEvent, dcm::DoorOpenRequest, profile::ProfileChangeEvent, select_chan, wait_for};
use barracuda_core::util::JsonStorage;
use barracuda_core::util::ObjectStorage;
use std::{sync::Arc, thread};
use components::Passageway;

use crate::components::serialization_types::*;
//use crate::components::outputcomponentbase::*;


mod components;

const MODULE_ID: u32 = 0x0D000000;

#[derive(Clone)]
enum PassagewayUpdate
{
     NewPassageway(u32),
     PassagewayUpdate(u32),
     DeletePassageway(u32)
}


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
    module_base         : barracuda_core::core::module_base::ModuleBase,
    bin_prof_rx         : Arc<GenericReceiver<ProfileChangeEvent>>,  
    input_rx            : Arc<GenericReceiver<InputEvent>>, 
    door_req_rx         : Arc<GenericReceiver<DoorOpenRequest>>,
    pway_change_rx      : Arc<GenericReceiver<PassagewayUpdate>>,
    passageways         : Vec<Passageway>,
    storage             : Shareable<JsonStorage<PassagewaySetting>>,
    trace               : trace_helper::TraceHelper
}

impl ADCM
{
    pub fn new(tracer: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        let mut result = Self
        {
            module_base         : barracuda_core::core::module_base::ModuleBase::new(MODULE_ID, tracer, chm),
            bin_prof_rx         : chm.get_receiver(),
            input_rx            : chm.get_receiver(),
            door_req_rx         : chm.get_receiver(),
            pway_change_rx      : chm.get_receiver(),
            passageways         : vec![],
            storage             : Shareable::new(JsonStorage::new("./passageways.txt".to_string())),
            trace               : trace_helper::TraceHelper::new("DCM/ADCM".to_string(), chm)
        };

        for setting in result.storage.lock().iter()
        {
            result.passageways.push(Passageway::new(setting.clone(), chm));
        }

        // let s = PassagewaySetting{
        //     id: 10,
        //     outputs: vec![OutputComponentSerialization::ElectricStrike(OutputComponentSetting {id: 24, operation_time: 3200})],
        //     inputs: vec![],
        //     access_points: vec![]
        // };
        // result.storage.lock().put_entry(s);
        // result.storage.lock().update_storage();

        return result;
    }

    pub fn init(&mut self)
    {
        let the_receiver = self.module_base.cfg_rx.clone(); 
        let hli_cb = Some(|| {

            let res = the_receiver.receive();
            let cfg::ConfigMessage::RegisterHandlers(cfg_holder) = res;
            let mut holder = cfg_holder.lock();

            let mut storage_new_setting = self.storage.clone();
            let mut storage_delete_setting = self.storage.clone();
            let pway_update_delete_tx = self.pway_change_rx.create_sender();
            let pway_update_update_tx = self.pway_change_rx.create_sender();

            holder.register_handler(FunctionType::Put, "adcm/passageway".to_string(), Handler!(|pway: PassagewaySetting|
                {
                    Self::process_passageway_setting(pway.clone(), &mut storage_new_setting);
                    pway_update_update_tx.send(PassagewayUpdate::PassagewayUpdate(pway.id));
                }));

            holder.register_handler(FunctionType::Delete, "adcm/passageway".to_string(), Handler!(|pway: PassagewaySetting|
                {
                    Self::process_delete_passageway(pway.clone(), &mut storage_delete_setting);
                    pway_update_delete_tx.send(PassagewayUpdate::DeletePassageway(pway.id));
                }));            
        });

        self.module_base.boot(Some(boot_noop), hli_cb);
    }

    fn process_passageway_setting(passageway: PassagewaySetting, storage: &mut Shareable<JsonStorage<PassagewaySetting>>)
    {
        let mut writeable_storage = storage.lock();
        let the_pway = writeable_storage.get_entry(|x|{x.id == passageway.id});
        if let Some(_existing_pway) = the_pway
        {
            //update existing passageqay... somehow!
        }

        writeable_storage.delete_entry(|x|{x.id == passageway.id});
        writeable_storage.put_entry(passageway);
        writeable_storage.update_storage();

    }

    fn process_delete_passageway(passageway: PassagewaySetting, storage: &mut Shareable<JsonStorage<PassagewaySetting>>)
    {
        let mut writeable_storage = storage.lock();
        let the_pway = writeable_storage.get_entry(|x|{x.id == passageway.id});
        if let Some(_existing_pway) = the_pway
        {
            // Kill off existing passsageway
        }

        writeable_storage.delete_entry(|x|{x.id == passageway.id});
        writeable_storage.update_storage();
    }

    pub fn run(&mut self) -> bool
    {        
        let queue_id = select_chan!(self.bin_prof_rx, self.input_rx, self.door_req_rx, self.pway_change_rx);
        match queue_id
        {
            0 => self.do_bin_prof_event(),
            1 => self.do_input_event(),
            2 => self.do_door_request(),
            3 => self.do_passageway_change_event(),
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
        self.trace.trace(format!("DoorRequest for accesspoint {}", door_request.access_point_id));
        for passageway in self.passageways.iter_mut()
        {            
            passageway.on_door_open_request(&door_request);
        }
    }

    fn load_passageway(&mut self, _pway_id: u32)
    {
        //let setting = self.storage.lock().get_entry(|x| x.id == pway_id);

    }

    fn do_passageway_change_event(&mut self)
    {
        let event = self.pway_change_rx.receive();

        match event
        {
            PassagewayUpdate::NewPassageway(id) => {self.load_passageway(id)},
            PassagewayUpdate::PassagewayUpdate(_id) => {},
            PassagewayUpdate::DeletePassageway(id) => {self.passageways.retain(|x| x.id != id)}
        }
    }
}