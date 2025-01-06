use barracuda_base_modules::events::LogEvent;

use crate::{DoorCommand, DoorEvent};
use super::{DoorStateContainer, DoorStateImpl, emergency::Emergency, normal_operation::NormalOperation};


#[derive(Copy, Clone)]
pub struct Blocked{}

impl DoorStateImpl for Blocked
{
    fn dispatch_door_event(self, passageway_id: u32, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
         match d
         {
             DoorEvent::ValidDoorOpenRequestSeen(ap_id, token) => {
                 commands.push(DoorCommand::ShowSignal(ap_id, barracuda_base_modules::sig::SigType::AccessDenied));
                 commands.push(DoorCommand::TriggerEvent(LogEvent::AccessDeniedDoorBlocked(passageway_id, token, ap_id)));
                }
             DoorEvent::BlockingContactDisengaged => {
                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorEnteredNormalOperation(passageway_id)));
                 return DoorStateContainer::NormalOp(NormalOperation{}, passageway_id);
                }
             DoorEvent::ReleaseSwitchEngaged => {return DoorStateContainer::Emergency(Emergency{}, passageway_id);}
             _ => {}
         }
         return DoorStateContainer::Blocked(self, passageway_id)
    }
}