
#[derive(Copy, Clone, PartialEq)]
pub enum SigType
{
    _Default,
    _AccessGranted,
    AccessDenied
}

#[derive(Copy, Clone)]
pub struct SigCommand
{
    pub access_point_id: u32,
    pub sig_type: SigType,
    pub duration: u32
}