use barracuda_base_modules::events::LogEvent;

use crate::{DoorCommand, DoorEvent};

use super::{blocked::Blocked, DoorStateContainer, DoorStateImpl, emergency::Emergency, normal_operation::NormalOperation};


#[derive(Copy, Clone)]
pub struct ReleasedPermanently{}

impl DoorStateImpl for ReleasedPermanently
{
    fn dispatch_door_event(self,passageway_id: u32, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::DoorOpenProfileInactive => {
                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorEnteredNormalOperation(passageway_id)));
                return DoorStateContainer::NormalOp(NormalOperation{}, passageway_id);                
            }
            DoorEvent::BlockingContactEngaged => {
                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorBlocked(passageway_id)));
                return DoorStateContainer::Blocked(Blocked{}, passageway_id);
            }
            DoorEvent::ReleaseSwitchEngaged => {
                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorEmergencyReleased(passageway_id)));
                return DoorStateContainer::Emergency(Emergency{}, passageway_id);
            }
            _ => {}
        }
        return DoorStateContainer::ReleasePerm(self, passageway_id)
    }
}
