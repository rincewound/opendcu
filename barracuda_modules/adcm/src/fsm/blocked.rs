use crate::{DoorCommand, DoorEvent};
use super::{DoorStateContainer, DoorStateImpl, emergency::Emergency, normal_operation::NormalOperation};


#[derive(Copy, Clone)]
pub struct Blocked{}

impl DoorStateImpl for Blocked
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
         match d
         {
             DoorEvent::ValidDoorOpenRequestSeen(ap_id) => {
                 commands.push(DoorCommand::ShowSignal(ap_id, barracuda_core::sig::SigType::AccessDenied));
                }
             DoorEvent::BlockingContactDisengaged => {return DoorStateContainer::NormalOp(NormalOperation{});}
             DoorEvent::ReleaseSwitchEngaged => {return DoorStateContainer::Emergency(Emergency{});}
             _ => {}
         }
         return DoorStateContainer::Blocked(self)
    }
}