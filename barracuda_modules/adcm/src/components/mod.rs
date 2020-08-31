use barracuda_core::core::broadcast_channel::*;
use barracuda_core::core::channel_manager::*;
use barracuda_core::dcm::*;
use barracuda_core::io::*;
use std::{sync::Arc};

use barracuda_core::profile::*;

mod electricstrike;
mod accessgranted;
mod outputcomponentbase;
mod framecontact;

#[derive(Copy, Clone)]
pub enum DoorEvent
{
    Opened,
    Closed,
    ForcedOpen,
    OpenTooLong,
    DoorOpenAlarm,
    ReleasedPermanently,
    ReleaseOnce,
    NormalOperation,
    Block
}

pub trait InputComponent: Send
{
    fn on_input_change(&mut self, event: &InputEvent);
    fn on_door_event(&mut self, event: DoorEvent);
}

pub trait OutputComponent: Send
{
    fn on_profile_change(&mut self, event: &ProfileChangeEvent);
    fn on_door_event(&mut self, event: DoorEvent);
}

pub trait VirtualComponent: Send
{
    fn on_door_event(&mut self, event: DoorEvent);
}


pub struct Passageway
{
    id: u32,
    door_open_profile_id: u32,
    //bin_prof_rx: Arc<GenericReceiver<ProfileChangeEvent>>,  // <-- Central dispatch here?
    //input_rx: Arc<GenericReceiver<InputEvent>>,             // <-- Central dispatch here?
    input_components: Vec<Box<dyn InputComponent>>,
    output_components: Vec<Box<dyn OutputComponent>>,
    virtual_components: Vec<Box<dyn VirtualComponent>>
}

impl Passageway
{
    pub fn on_profile_change(&mut self, event: &ProfileChangeEvent)
    {
        for v in self.output_components.iter_mut()
        {
            v.on_profile_change(event);
        }

        // ToDo: if the profile is our door open profile, we have
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
    }

    pub fn on_input_change(&mut self, event: &InputEvent)
    {
        for v in self.input_components.iter_mut()
        {
            v.on_input_change(event);
        }
    }

    pub fn handle_door_event(&mut self, event: DoorEvent)
    {
        for v in self.output_components.iter_mut()
        {
            v.on_door_event(event);
        }

        for v in self.input_components.iter_mut()
        {
            v.on_door_event(event);
        }

        for v in self.virtual_components.iter_mut()
        {
            v.on_door_event(event);
        }   
    }

    pub fn on_door_open_request(&mut self, request: &DoorOpenRequest)
    {
        // Check if AP belongs to this door

        // Check doorstate: If we're blocked, signal this, otherwise
        // signal access granted here and release the door.

        self.handle_door_event(DoorEvent::ReleaseOnce);
    }
}
