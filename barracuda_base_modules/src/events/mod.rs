#[derive( Clone, Debug, PartialEq)]
pub enum LogEvent
{
    AccessGranted(u32, Vec<u8>, u32),                  // pwayid, token, ap id
    AccessDeniedTimezoneViolated(u32, Vec<u8>, u32),   // pwayid, token, ap id
    AccessDeniedTokenUnknown(u32, Vec<u8>, u32),       // pwayid, token, ap id
    AccessDeniedDoorBlocked(u32, Vec<u8>, u32),        // pwayid, token, ap id

    DoorEmergencyReleased(u32),      // pwayid
    DoorEnteredNormalOperation(u32), // pwayid
    DoorPermantlyReleased(u32),      // pwayid
    DoorReleasedOnce(u32),  // pwayid
    DoorBlocked(u32),       // pwayid
    DoorForcedOpen(u32),    // pwayid
    DoorOpenTooLong(u32),   // pwayid
    DoorClosedAgain(u32),   // pwayid
}