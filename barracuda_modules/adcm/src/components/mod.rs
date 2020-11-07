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




// #[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub enum DoorEvent
// {
//     Opened,                 // Triggered by FC
//     Closed,                 // Triggered by FC
//     ForcedOpen,             // Triggered by FC
//     _OpenTooLong,           // Generated by TBD (alarm generator of some sort)
//     DoorOpenAlarm,          // Generated by TBD (alarm generator of some sort)
//     ReleasedPermanently,    // Generated by Profile Change
//     ReleaseOnce,            // Generated on Access Request
//     NormalOperation,        // Generated by Profile Change
//     AccessAllowed,          // Generated by a valid booking *or* a doorhandle. Shall suppress alarms of the FC
//     Block,
//     EmergencyRelease
// }

pub trait InputComponent: Send
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>);
    //fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
}

pub trait OutputComponent: Send
{
    fn on_profile_change(&mut self, event: &ProfileChangeEvent, generated_events: &mut Vec<DoorEvent>);
    //fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
    fn on_door_command(&mut self, command: DoorCommand);
}
pub trait VirtualComponent: Send
{
    fn on_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>);
}

