use std::sync::{Arc};

use barracuda_base_modules::{dcm::DoorOpenRequest, events::LogEvent, io::InputEvent, profile::{ProfileChangeEvent, ProfileState}, sig::{SigCommand, SigType}};
use barracuda_core::{core::{
        broadcast_channel::GenericSender, 
        channel_manager::ChannelManager, 
        timer::Timer}, core::shareable::Shareable, trace::trace_helper::TraceHelper};
use crate::{DoorCommand, DoorEvent, components::output_components::accessgranted::AccessGranted, components::{InputComponent, OutputComponent, VirtualComponent, output_components::{alarmrelay::AlarmRelay, electricstrike::ElectricStrike}, serialization_types::InputComponentSerialization, serialization_types::{OutputComponentSerialization, PassagewaySetting}}};

use crate::fsm::*;
use crate::fsm::normal_operation::NormalOperation;

const DEFAULT_RELEASE_TIME: u64 = 5000;

pub struct Passageway
{
    pub id: u32,
    door_open_profile_id: u32,
    access_points: Vec<u32>,
    input_components: Vec<Box<dyn InputComponent>>,
    output_components: Shareable<Vec<Box<dyn OutputComponent>>>,
    _virtual_components: Vec<Box<dyn VirtualComponent>>,
    pending_events: Vec<DoorEvent>,
    sig_tx: GenericSender<SigCommand>,
    log_tx: GenericSender<LogEvent>,
    trace: TraceHelper,
    door_fsm: Shareable<crate::fsm::DoorStateContainer>,
    auto_event_timer: Arc<Timer>,
    auto_switch_normal_timer: Option<Arc<bool>>,
    door_open_too_long_timer: Option<Arc<bool>>,
    alarm_time: u64,
    release_time: u64,
    channel_manager: barracuda_core::core::channel_manager::ChannelManager
}

impl Passageway
{
    fn load_input_components(components: &Vec<InputComponentSerialization>) -> Vec<Box<dyn InputComponent>>
    {
        let mut deserialized_components: Vec<Box<dyn InputComponent>> = vec![];
        for component in components.iter()
        {
            let the_object: Box<dyn InputComponent>;
            match component
            {
                InputComponentSerialization::FrameContact(setting) => {the_object = Box::new(*setting)}
                InputComponentSerialization::DoorOpenerKey(dooropenerkey) => { the_object = Box::new(*dooropenerkey)}
                InputComponentSerialization::DoorHandle(doorhandle) => { the_object = Box::new(*doorhandle)}
                InputComponentSerialization::ReleaseContact(releasecontact) => { the_object = Box::new(*releasecontact)}
            }
            deserialized_components.push(the_object);
        }
        deserialized_components
    }

    fn load_output_components(components: &Vec<OutputComponentSerialization>, chm: &mut ChannelManager) -> Vec<Box<dyn OutputComponent>>
    {
        let mut deserialized_components: Vec<Box<dyn OutputComponent>> = vec![];
        for component in components.iter()
        { 
            let the_object: Box<dyn OutputComponent>;
            match component
            {
                OutputComponentSerialization::ElectricStrike(setting) => {the_object = Box::new(ElectricStrike::from_setting(*setting, chm))}
                OutputComponentSerialization::AccessGranted(setting) => {the_object = Box::new(AccessGranted::from_setting(*setting, chm))}                
                OutputComponentSerialization::AlarmRelay(output_id) => {the_object = Box::new(AlarmRelay::new(*output_id, chm))}                
            }
            deserialized_components.push(the_object);
        }
        deserialized_components
    }

    fn find_release_time(settings: &PassagewaySetting) -> u64
    {
        for i in settings.outputs.iter()
        {
            match i
            {
                OutputComponentSerialization::ElectricStrike(strike) => return strike.operation_time,
                _ => {}
            }
        }
        // TBD: Should we panic in case we have no strike attached to the door?
        return DEFAULT_RELEASE_TIME;
    }

    pub fn new(settings: PassagewaySetting, chm: &mut ChannelManager) -> Self
    {
        Self 
        {
            id: settings.id,
            access_points: settings.access_points.clone(),
            door_open_profile_id: 0,
            input_components: Passageway::load_input_components(&settings.inputs),
            output_components: Shareable::new(Passageway::load_output_components(&settings.outputs, chm)),
            _virtual_components: vec![],
            pending_events: vec![],
            sig_tx: chm.get_sender(),
            log_tx: chm.get_sender(),
            trace: TraceHelper::new(format!("ADCM/PW{}", settings.id), chm),            
            door_fsm: Shareable::new(DoorStateContainer::NormalOp(NormalOperation{}, settings.id)),
            auto_event_timer: Timer::new(),
            auto_switch_normal_timer: None,
            door_open_too_long_timer: None,
            alarm_time: settings.alarm_time,
            release_time: Passageway::find_release_time(&settings),
            channel_manager: chm.clone()
        }
    }

    pub fn apply_settings(&mut self, settings: PassagewaySetting)
    {
        self.input_components = Passageway::load_input_components(&settings.inputs);
        self.output_components = Shareable::new(Passageway::load_output_components(&settings.outputs, &mut self.channel_manager));
        self.alarm_time = settings.alarm_time;
        self.release_time = Passageway::find_release_time(&settings);

        // We don't change the doorstate at all. This way we can change the settings
        // without the user noticing or having to restart the device. Note that this 
        // can cause a problem if the door is actually open when the settings change
        // and an existing framecontact is removed. In this case the door will entry
        // alarm state at some point and never recover, because it no longer "sees"
        // the door closed event. To make sure the door returns to normal in all
        // cases we arm the "return to default state" timer. The events generated
        // by that timer are ignored by all but the door open state. This ensures,
        // that the door returns to NormalOp at most release_time seconds after
        // the confiuration change.
        if self.auto_switch_normal_timer.is_none()
        {
            self.auto_switch_normal_timer = Some(self.arm_timer(DoorEvent::DoorTimerExpired, self.release_time));
        }
    }

    pub fn on_profile_change(&mut self, event: &ProfileChangeEvent)
    {
        // if the profile is our door open profile, we have
        // to adjust the doorstate here as well
        if event.profile_id == self.door_open_profile_id
        {            
            if event.profile_state == ProfileState::Active
            {
                self.handle_door_event(DoorEvent::DoorOpenProfileActive);
            }
            if event.profile_state == ProfileState::Inactive
            {
                self.handle_door_event(DoorEvent::DoorOpenProfileInactive);
            }
        }
        self.do_events();
    }

    pub fn on_input_change(&mut self, event: &InputEvent)
    {
        for v in self.input_components.iter_mut()
        {
            v.on_input_change(event, &mut self.pending_events);
        }
        self.do_events();
    }

    pub fn handle_door_event(&mut self, event: DoorEvent)
    {
        let mut generated_commands : Vec<DoorCommand>;
        generated_commands = vec![];
        Passageway::inject_door_event(event, &mut self.door_fsm, &mut generated_commands);
        self.do_door_commands(generated_commands);   
    }

    fn inject_door_event(door_event: DoorEvent, current_door_state: &mut Shareable<DoorStateContainer>, generated_commands: &mut Vec<DoorCommand>)
    {
        let next_state : DoorStateContainer;
        let mut fsm_lcked = current_door_state.lock();
        let fsm = *fsm_lcked;
        match fsm
        {
            DoorStateContainer::NormalOp(op, id) => { next_state = op.dispatch_door_event(id,door_event, generated_commands);}
            DoorStateContainer::ReleasedOnce(op, id) =>  { next_state = op.dispatch_door_event(id,door_event, generated_commands);}
            DoorStateContainer::ReleasePerm(op, id) => {next_state = op.dispatch_door_event(id, door_event, generated_commands);}
            DoorStateContainer::Blocked(op, id) => {next_state = op.dispatch_door_event(id, door_event, generated_commands);}
            DoorStateContainer::Emergency(op, id) => {next_state = op.dispatch_door_event(id, door_event, generated_commands);}
        }

        *fsm_lcked = next_state;
        drop(fsm_lcked);      
    }

    fn arm_timer(&self, event_to_generate: DoorEvent, timeout: u64) -> Arc<bool>
    {
        let mut fsm_clone = self.door_fsm.clone();
        let mut generated_commands : Vec<DoorCommand>;
        let mut outputs = self.output_components.clone();
        generated_commands = vec![];                 
        return self.auto_event_timer.schedule( Box::new( move|| {
                Passageway::inject_door_event(event_to_generate, &mut fsm_clone, &mut generated_commands);
                /* Attention: This will need some refactoring at some point, as this solution will only
                   execute commands that influence the behavior of outputs. Commands that are to be executed
                   by the Passageway will be ignored. 
                */
                Passageway::do_doorcommands_for_outputs(&mut outputs, generated_commands);
            }), timeout);
    }

    fn do_door_commands(&mut self, commands: Vec<DoorCommand>)
    {
        for cmd in commands.iter()
        {
            match cmd
            {
                DoorCommand::ArmDoorOpenTooLongAlarm => {
                    self.auto_switch_normal_timer = Some(self.arm_timer(DoorEvent::DoorOpenTooLong, self.alarm_time));
                },
                DoorCommand::DisarmDoorOpenTooLongAlarm => {
                    self.door_open_too_long_timer = None
                },
                DoorCommand::DisarmAutoswitchToNormal => {
                    self.auto_switch_normal_timer = None
                },
                DoorCommand::ArmAutoswitchToNormal => {
                    self.auto_switch_normal_timer = Some(self.arm_timer(DoorEvent::DoorTimerExpired, self.release_time));
                },
                DoorCommand::ShowSignal(ap_id, sig) =>
                {
                    // ToDo: Configurabale signal times!
                    self.send_signal_command(*ap_id, *sig, 3500);
                }
                DoorCommand::TriggerEvent(log_event) =>
                {
                    self.log_tx.send(log_event.clone());
                }
                _ => {

                    for output in self.output_components.lock().iter_mut()
                    {
                        let the_cmd = cmd.clone();
                        output.on_door_command(the_cmd);
                    }
                }
            }
        }
    }

    fn do_doorcommands_for_outputs(outputs: &mut Shareable<Vec<Box<dyn OutputComponent>>>, commands: Vec<DoorCommand>)
    {
        for cmd in commands.iter()
        {
            for output in outputs.lock().iter_mut()
            {
                output.on_door_command(cmd.clone());
            }  
        }    
    }

    pub fn on_door_open_request(&mut self, request: &DoorOpenRequest)
    {
        // Check if AP belongs to this door
        if !self.access_points.contains(&request.access_point_id)
        {
            return;
        }

        self.trace.trace_str("Release once.");
        self.handle_door_event(DoorEvent::ValidDoorOpenRequestSeen(request.access_point_id, request.identification_token.clone()));
    }

    fn send_signal_command(&self, access_point_id: u32, sigtype: SigType, duration: u32)
    {
        let sig = SigCommand {
            access_point_id,
            sig_type: sigtype, 
            duration
        };

        self.sig_tx.send(sig); 
    }

    fn do_events(&mut self)
    {
        while let Some(evt) = self.pending_events.pop()
        {
            self.handle_door_event(evt);
        }
    }
}
