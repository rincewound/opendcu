use barracuda_base_modules::events::LogEvent;

use crate::{DoorCommand, DoorEvent};
use super::{DoorStateContainer, DoorStateImpl, normal_operation::NormalOperation};


#[derive(Copy, Clone)]
pub struct Emergency{}

impl DoorStateImpl for Emergency
{
    fn dispatch_door_event(self, passageway_id: u32, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::ReleaseSwitchDisengaged => {
                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorEnteredNormalOperation(passageway_id)));
                return DoorStateContainer::NormalOp(NormalOperation{}, passageway_id);
            }
            _ => {}
        }
        return DoorStateContainer::Emergency(self, passageway_id)
    }

}