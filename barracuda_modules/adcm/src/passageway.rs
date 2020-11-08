use std::sync::{Arc};

use barracuda_core::{core::{channel_manager::ChannelManager, broadcast_channel::GenericSender}, core::timer::Timer, dcm::DoorOpenRequest, io::InputEvent, core::shareable::Shareable, profile::{ProfileChangeEvent, ProfileState}, sig::SigCommand, sig::SigType, trace::trace_helper::TraceHelper};

use crate::{DoorCommand, DoorEvent, components::{InputComponent, OutputComponent, VirtualComponent, accessgranted::AccessGranted, alarmrelay::AlarmRelay, electricstrike::ElectricStrike, serialization_types::InputComponentSerialization, serialization_types::{OutputComponentSerialization, PassagewaySetting}}};

use crate::fsm::*;

pub struct Passageway
{
    pub id: u32,
    door_open_profile_id: u32,
    access_points: Vec<u32>,
    input_components: Vec<Box<dyn InputComponent>>,
    output_components: Shareable<Vec<Box<dyn OutputComponent>>>,
    virtual_components: Vec<Box<dyn VirtualComponent>>,
    pending_events: Vec<DoorEvent>,
    sig_tx:  GenericSender<SigCommand>,
    trace: TraceHelper,
    door_fsm: Shareable<crate::fsm::DoorStateContainer>,
    auto_event_timer: Arc<Timer>,
    auto_switch_normal_timer: Option<Arc<bool>>
}

impl Passageway
{
    fn load_input_components(components: Vec<InputComponentSerialization>) -> Vec<Box<dyn InputComponent>>
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

    fn load_output_components(components: Vec<OutputComponentSerialization>, chm: &mut ChannelManager) -> Vec<Box<dyn OutputComponent>>
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

    pub fn new(settings: PassagewaySetting, chm: &mut ChannelManager) -> Self
    {
        Self 
        {
            id: settings.id,
            access_points: settings.access_points,
            door_open_profile_id: 0,
            input_components: Passageway::load_input_components(settings.inputs),
            output_components: Shareable::new(Passageway::load_output_components(settings.outputs, chm)),
            virtual_components: vec![],
            pending_events: vec![],
            sig_tx: chm.get_sender(),
            trace: TraceHelper::new(format!("ADCM/PW{}", settings.id), chm),            
            door_fsm: Shareable::new(DoorStateContainer::NormalOp(NormalOperation{})),
            auto_event_timer: Timer::new(),
            auto_switch_normal_timer: None
        }
    }

    pub fn on_profile_change(&mut self, event: &ProfileChangeEvent)
    {
        for v in self.output_components.lock().iter_mut()
        {
            v.on_profile_change(event, &mut self.pending_events);
        }

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
            DoorStateContainer::NormalOp(op) => { next_state = op.dispatch_door_event(door_event, generated_commands);}
            DoorStateContainer::ReleasedOnce(op) =>  { next_state = op.dispatch_door_event(door_event, generated_commands);}
            DoorStateContainer::ReleasePerm(op) => {next_state = DoorStateContainer::NormalOp(NormalOperation{});}
            DoorStateContainer::Blocked(op) => {next_state = DoorStateContainer::NormalOp(NormalOperation{});}
            DoorStateContainer::Emergency(op) => {next_state = DoorStateContainer::NormalOp(NormalOperation{});}
        }

        *fsm_lcked = next_state;
        drop(fsm_lcked);      
    }

    fn do_door_commands(&mut self, commands: Vec<DoorCommand>)
    {
        for cmd in commands.iter()
        {
            match cmd
            {
                DoorCommand::ArmDoorOpenTooLongAlarm => {
                },
                DoorCommand::DisarmDoorOpenTooLongAlarm => {
                    self.auto_switch_normal_timer = None
                },
                DoorCommand::DisarmAutoswitchToNormal => {
                    self.auto_switch_normal_timer = None
                },
                DoorCommand::ArmAutoswitchToNormal => {
                    let mut fsm_clone = self.door_fsm.clone();
                    let mut generated_commands : Vec<DoorCommand>;
                    let mut outputs = self.output_components.clone();
                    generated_commands = vec![];       
                    // ToDo: Obtain Strike Timeout for timer here!             
                    self.auto_switch_normal_timer =  Some(self.auto_event_timer.schedule( Box::new( move|| {
                            Passageway::inject_door_event(DoorEvent::DoorTimerExpired, &mut fsm_clone, &mut generated_commands);
                            /* Attention: This will need some refactoring at some point, as this solution will only
                               execute commands that influence the behavior of outputs. Commands that are to be executed
                               by the Passageway will be ignored. 
                            */
                            Passageway::do_doorcommands_for_outputs(&mut outputs, generated_commands);
                        }), 5000));
                },
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

        // Check doorstate: If we're blocked, signal this, otherwise
        // signal access granted here and release the door.
        self.trace.trace_str("Release once.");
        self.handle_door_event(DoorEvent::ValidDoorOpenRequestSeen);
        self.send_signal_command(request.access_point_id, SigType::AccessGranted, 3000);
    }

    fn send_signal_command(&self, access_point_id: u32, sigtype: SigType, duration: u32)
    {
        let sig = SigCommand {
            access_point_id: access_point_id,
            sig_type: sigtype, 
            duration: duration
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
