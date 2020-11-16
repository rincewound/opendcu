use crate::{DoorCommand, DoorEvent};
use super::{DoorStateContainer, DoorStateImpl, NormalOperation::NormalOperation};


#[derive(Copy, Clone)]
pub struct Emergency{}

impl DoorStateImpl for Emergency
{
    fn dispatch_door_event(self, d: DoorEvent, _commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::ReleaseSwitchDisengaged => {return DoorStateContainer::NormalOp(NormalOperation{});}
            _ => {}
        }
        return DoorStateContainer::Emergency(self)
    }
}