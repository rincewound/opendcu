use crate::{DoorCommand, DoorEvent};

pub mod normal_operation;
mod released_once;
mod released_permanently;
mod blocked;
mod emergency;


#[derive(Copy, Clone)]
pub enum DoorStateContainer
{
    NormalOp(normal_operation::NormalOperation),
    ReleasedOnce(released_once::ReleasedOnce),
    ReleasePerm(released_permanently::ReleasedPermanently),
    Blocked(blocked::Blocked),
    Emergency(emergency::Emergency)
}


// Use Enum dispatch here!
pub trait DoorStateImpl
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer;
}