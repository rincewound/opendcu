
#[derive(Clone)]
pub struct DoorOpenRequest
{
    pub access_point_id: u32,
    pub identification_token: Vec<u8>
}

// pub enum DoorControlCommands
// {
//     DoorOpenRequest,
//     BarDoorRequest,
//     PermanentReleaseRequest
// }

pub mod trivial;