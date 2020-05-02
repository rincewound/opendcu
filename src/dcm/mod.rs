
#[derive(Clone, Copy)]
pub struct DoorOpenRequest
{
    pub access_point_id: u32
}

pub enum DoorControlCommands
{
    DoorOpenRequest,
    BarDoorRequest,
    PermanentReleaseRequest
}