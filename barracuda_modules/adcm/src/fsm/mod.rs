use crate::{DoorCommand, DoorEvent};

pub mod NormalOperation;
mod ReleasedOnce;
mod ReleasedPermanently;
mod Blocked;
mod Emergency;


#[derive(Copy, Clone)]
pub enum DoorStateContainer
{
    NormalOp(NormalOperation::NormalOperation),
    ReleasedOnce(ReleasedOnce::ReleasedOnce),
    ReleasePerm(ReleasedPermanently::ReleasedPermanently),
    Blocked(Blocked::Blocked),
    Emergency(Emergency::Emergency)
}


// Use Enum dispatch here!
pub trait DoorStateImpl
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer;
}