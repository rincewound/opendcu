use barracuda_core::{trace::trace_helper::TraceHelper, core::{broadcast_channel::{GenericSender}, channel_manager::*}, sig::{SigCommand, SigType}};
use barracuda_core::dcm::*;
use barracuda_core::io::*;
use barracuda_core::profile::*;

use serde::{Deserialize, Serialize};

mod electricstrike;
mod accessgranted;
pub mod outputcomponentbase;
mod framecontact;
mod dooropenerkey;
mod doorhandle;
mod releasecontact;
mod blockingcontact;

pub mod serialization_types;

use serialization_types::*;

use self::{accessgranted::AccessGranted, electricstrike::ElectricStrike};

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum DoorEvent
{
    Opened,                 // Triggered by FC
    Closed,                 // Triggered by FC
    ForcedOpen,             // Triggered by FC
    _OpenTooLong,           // Generated by TBD (alarm generator of some sort)
    DoorOpenAlarm,          // Generated by TBD (alarm generator of some sort)
    ReleasedPermanently,    // Generated by Profile Change
    ReleaseOnce,            // Generated on Access Request
    NormalOperation,        // Generated by Profile Change
    AccessAllowed,          // Generated by a valid booking *or* a doorhandle. Shall suppress alarms of the FC
    Block
}

pub trait InputComponent: Send
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>);
    fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
}

pub trait OutputComponent: Send
{
    fn on_profile_change(&mut self, event: &ProfileChangeEvent, generated_events: &mut Vec<DoorEvent>);
    fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
}
pub trait VirtualComponent: Send
{
    fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
}


pub struct Passageway
{
    pub id: u32,
    door_open_profile_id: u32,
    access_points: Vec<u32>,
    input_components: Vec<Box<dyn InputComponent>>,
    output_components: Vec<Box<dyn OutputComponent>>,
    virtual_components: Vec<Box<dyn VirtualComponent>>,
    pending_events: Vec<DoorEvent>,
    sig_tx:  GenericSender<SigCommand>,
    trace: TraceHelper
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
            output_components: Passageway::load_output_components(settings.outputs, chm),
            virtual_components: vec![],
            pending_events: vec![],
            sig_tx: chm.get_sender(),
            trace: TraceHelper::new(format!("ADCM/PW{}", settings.id), chm)
        }
    }

    pub fn on_profile_change(&mut self, event: &ProfileChangeEvent)
    {
        for v in self.output_components.iter_mut()
        {
            v.on_profile_change(event, &mut self.pending_events);
        }

        // if the profile is our door open profile, we have
        // to adjust the doorstate here as well
        if event.profile_id == self.door_open_profile_id
        {
            if event.profile_state == ProfileState::Active
            {
                self.handle_door_event(DoorEvent::ReleasedPermanently);
            }
            if event.profile_state == ProfileState::Inactive
            {
                self.handle_door_event(DoorEvent::NormalOperation);
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
        for v in self.output_components.iter_mut()
        {
            v.on_door_event(event, &mut self.pending_events);
        }

        for v in self.input_components.iter_mut()
        {
            v.on_door_event(event, &mut self.pending_events);
        }

        for v in self.virtual_components.iter_mut()
        {
            v.on_door_event(event, &mut self.pending_events);
        }
        self.do_events();   
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
        self.handle_door_event(DoorEvent::ReleaseOnce);
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
