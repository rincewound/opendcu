use barracuda_core::{core::{channel_manager::*}};

use barracuda_core::io::*;
use barracuda_core::profile::*;
use crate::{DoorCommand, DoorEvent};

pub mod electricstrike;
pub mod accessgranted;
pub mod outputcomponentbase;
pub mod framecontact;
pub mod dooropenerkey;
pub mod doorhandle;
pub mod releasecontact;
pub mod blockingcontact;
pub mod alarmrelay;

pub mod serialization_types;

pub trait InputComponent: Send
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>);    
}

pub trait OutputComponent: Send
{
    fn on_profile_change(&mut self, event: &ProfileChangeEvent, generated_events: &mut Vec<DoorEvent>);
    fn on_door_command(&mut self, command: DoorCommand);
}
pub trait VirtualComponent: Send
{
    fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
}

