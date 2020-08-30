use barracuda_core::core::broadcast_channel::*;
use barracuda_core::core::channel_manager::*;
use barracuda_core::dcm::*;
use barracuda_core::io::*;
use std::{sync::Arc};

use barracuda_core::profile::*;

mod electricstrike;
mod accessgranted;
mod outputcomponentbase;

enum DoorEvent
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

pub trait InputComponent
{
    fn on_input_change(&mut self, event: &InputEvent);
    fn on_door_event(&mut self, event: DoorEvent);
}

pub trait OutputComponent
{
    fn on_profile_change(&mut self, event: &ProfileChangeEvent);
    fn on_door_event(&mut self, event: DoorEvent);
}


struct Passageway
{
    id: u32,
    door_open_profile_id: u32,
    //bin_prof_rx: Arc<GenericReceiver<ProfileChangeEvent>>,  // <-- Central dispatch here?
    //input_rx: Arc<GenericReceiver<InputEvent>>,             // <-- Central dispatch here?
    input_components: Vec<Box<dyn InputComponent>>,
    output_components: Vec<Box<dyn OutputComponent>>
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
    }

    pub fn on_input_change(&mut self, event: &InputEvent)
    {
        for v in self.input_components.iter_mut()
        {
            v.on_input_change(event);
        }
    }

    pub fn on_door_open_request(&mut self, request: DoorOpenRequest)
    {
        // Check doorstate: If we're blocked, signal this, otherwise
        // signal access granted here and release the door.
    }
}