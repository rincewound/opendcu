use crate::{DoorCommand, DoorEvent};

use super::{Blocked::Blocked, DoorStateContainer, DoorStateImpl, Emergency::Emergency, NormalOperation::NormalOperation};


#[derive(Copy, Clone)]
pub struct ReleasedPermanently{}

impl DoorStateImpl for ReleasedPermanently
{
    fn dispatch_door_event(self, d: DoorEvent, _commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::DoorOpenProfileInactive => {return DoorStateContainer::NormalOp(NormalOperation{});}
            DoorEvent::BlockingContactEngaged => {return DoorStateContainer::Blocked(Blocked{});}
            DoorEvent::ReleaseSwitchEngaged => {return DoorStateContainer::Emergency(Emergency{});}
            _ => {}
        }
        return DoorStateContainer::ReleasePerm(self)
    }
}
