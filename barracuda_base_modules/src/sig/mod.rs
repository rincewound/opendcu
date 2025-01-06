
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SigType
{
    _Default,
    AccessGranted,
    AccessDenied,
}

#[derive(Copy, Clone)]
pub struct SigCommand
{
    pub access_point_id: u32,
    pub sig_type: SigType,
    pub duration: u32
}

pub mod sigctrl;