use barracuda_core::io::*;
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
    fn on_door_command(&mut self, command: DoorCommand);
}
pub trait VirtualComponent: Send
{
    fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
}

