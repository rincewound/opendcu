use barracuda_core::core::channel_manager::*;
use barracuda_core::dcm::*;
use barracuda_core::io::*;

use barracuda_core::profile::*;

mod electricstrike;
mod accessgranted;
mod outputcomponentbase;
mod framecontact;

#[derive(Copy, Clone, PartialEq)]
pub enum DoorEvent
{
    Opened,
    Closed,
    ForcedOpen,
    _OpenTooLong,
    DoorOpenAlarm,
    ReleasedPermanently,
    ReleaseOnce,
    NormalOperation,
    _Block
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
    id: u32,
    door_open_profile_id: u32,
    input_components: Vec<Box<dyn InputComponent>>,
    output_components: Vec<Box<dyn OutputComponent>>,
    virtual_components: Vec<Box<dyn VirtualComponent>>,
    pending_events: Vec<DoorEvent>,
}

impl Passageway
{
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

        // Check doorstate: If we're blocked, signal this, otherwise
        // signal access granted here and release the door.

        self.handle_door_event(DoorEvent::ReleaseOnce);
    }

    fn do_events(&mut self)
    {
        while let Some(evt) = self.pending_events.pop()
        {
            self.handle_door_event(evt);
        }
    }
}
