#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LogEvent
{
    AccessGranted,                  // pwayid, token, ap id
    AccessDeniedTimezoneViolated,   // pwayid, token, ap id
    AccessDeniedTokenUnknown,       // pwayid, token, ap id

    AccessDeniedDoorBlocked,        // pwayid, token, ap id

    DoorEmergencyReleased(u32),      // pwayid
    DoorEnteredNormalOperation(u32), // pwayid
    DoorPermantlyReleased(u32),      // pwayid
    DoorReleasedOnce(u32),  // pwayid
    DoorBlocked(u32),       // pwayid
    DoorForcedOpen(u32),    // pwayid
    DoorOpenTooLong(u32),   // pwayid
    DoorClosedAgain(u32),   // pwayid
}