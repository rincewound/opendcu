use barracuda_base_modules::{cfg::{self, cfgholder::FunctionType}, dcm::DoorOpenRequest, events::LogEvent, io::{InputEvent, OutputState}, modulebase::ModuleBase, profile::ProfileChangeEvent, sig::SigType};
use barracuda_base_modules::Handler;
use barracuda_core::core::{broadcast_channel::GenericReceiver, channel_manager::*, shareable::Shareable};
use barracuda_core::core::{bootstage_helper::*, event::DataEvent};

use barracuda_core::trace::*;
use barracuda_core::{select_chan, wait_for};
use barracuda_core::util::JsonStorage;
use barracuda_core::util::ObjectStorage;
use std::{sync::Arc, thread};
use crate::components::serialization_types::*;
use passageway::Passageway;


mod components;
mod fsm;
mod passageway;

const MODULE_ID: u32 = 0x0D000000;

#[derive(Clone)]
enum PassagewayUpdate
{
     PassagewayUpdate(u32),
     DeletePassageway(u32)
}

#[derive(PartialEq)]
pub enum DoorEvent
{
    ValidDoorOpenRequestSeen(u32, Vec<u8>), //ap id, access token
    Opened,
    Closed,
    DoorOpenProfileActive,
    DoorOpenProfileInactive,
    BlockingContactEngaged,
    BlockingContactDisengaged,
    ReleaseSwitchEngaged,
    ReleaseSwitchDisengaged,
    DoorOpenerKeyTriggered,
    DoorHandleTriggered,
    DoorOpenTooLong,
    DoorTimerExpired
}

// Generated by the FSM, consumed by outputs. The inputs
// don't consume doorevents at all
#[derive(Clone, Debug, PartialEq)]
pub enum DoorCommand
{
    ToggleElectricStrike(OutputState),
    ToggleElectricStrikeTimed(OutputState),
    ToggleAccessAllowed(OutputState),
    ToggleAlarmRelay(OutputState),
    ArmDoorOpenTooLongAlarm,
    DisarmDoorOpenTooLongAlarm,
    ArmAutoswitchToNormal,
    DisarmAutoswitchToNormal,
    ShowSignal(u32, SigType),
    TriggerEvent(LogEvent)
}

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("DCM/ADCM".to_string(), chm);
    let mut chmclone = chm.clone();
    thread::spawn(move || {        
        let mut adcm = ADCM::new(tracer, &mut chmclone);
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
    module_base         : ModuleBase,
    bin_prof_rx         : GenericReceiver<ProfileChangeEvent>,  
    input_rx            : GenericReceiver<InputEvent>, 
    door_req_rx         : GenericReceiver<DoorOpenRequest>,
    pway_change_rx      : GenericReceiver<PassagewayUpdate>,
    passageways         : Vec<Passageway>,
    storage             : Shareable<JsonStorage<PassagewaySetting>>,
    trace               : trace_helper::TraceHelper,
    channel_manager     : ChannelManager
}

impl ADCM
{
    pub fn new(tracer: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        let mut result = Self
        {
            module_base         : ModuleBase::new(MODULE_ID, tracer, chm),
            bin_prof_rx         : chm.get_receiver(),
            input_rx            : chm.get_receiver(),
            door_req_rx         : chm.get_receiver(),
            pway_change_rx      : chm.get_receiver(),
            passageways         : vec![],
            storage             : Shareable::new(JsonStorage::new("./passageways.txt".to_string())),
            trace               : trace_helper::TraceHelper::new("DCM/ADCM".to_string(), chm),
            channel_manager     : chm.clone()
        };

        for setting in result.storage.lock().iter()
        {
            result.passageways.push(Passageway::new(setting.clone(), chm));
        }

        return result;
    }

    pub fn init(&mut self)
    {
        let the_receiver = self.module_base.cfg_rx.clone_receiver(); 
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
        writeable_storage.delete_entry(|x|{x.id == passageway.id});
        writeable_storage.put_entry(passageway);
        writeable_storage.update_storage();
    }

    fn process_delete_passageway(passageway: PassagewaySetting, storage: &mut Shareable<JsonStorage<PassagewaySetting>>)
    {
        let mut writeable_storage = storage.lock();
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

    fn update_passageway(&mut self, pway_id: u32)
    {
        if let Some(setting) = self.storage.lock().get_entry(|x| x.id == pway_id)
        {
            if let Some(pway) = self.passageways.iter_mut().find(|x| x.id == pway_id)
            {
                // We can *just* change the components of the passageway, as the complete state is externalized
                // and the pway will immediately start to react according to the setting.
                pway.apply_settings(setting);
            }
            else
            {
                self.passageways.push(Passageway::new(setting.clone(), &mut self.channel_manager));
            }
        }
        
    }

    fn do_passageway_change_event(&mut self)
    {
        let event = self.pway_change_rx.receive();

        match event
        {
            PassagewayUpdate::PassagewayUpdate(id) => {self.update_passageway(id)},
            PassagewayUpdate::DeletePassageway(id) => {self.passageways.retain(|x| x.id != id)}
        }
    }
}