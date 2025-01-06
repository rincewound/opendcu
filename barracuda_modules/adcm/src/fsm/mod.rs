use crate::{DoorCommand, DoorEvent};

pub mod normal_operation;
mod released_once;
mod released_permanently;
mod blocked;
mod emergency;


#[derive(Copy, Clone)]
pub enum DoorStateContainer
{
    NormalOp(normal_operation::NormalOperation, u32),
    ReleasedOnce(released_once::ReleasedOnce, u32),
    ReleasePerm(released_permanently::ReleasedPermanently, u32),
    Blocked(blocked::Blocked, u32),
    Emergency(emergency::Emergency, u32)
}


// Use Enum dispatch here!
pub trait DoorStateImpl
{
    fn dispatch_door_event(self, passageway_id: u32, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer;
}