use barracuda_core::io::*;
use barracuda_core::profile::*;
use crate::{DoorCommand, DoorEvent};

pub mod output_components;
pub mod input_components;

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

