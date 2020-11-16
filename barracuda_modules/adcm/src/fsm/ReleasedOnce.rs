use barracuda_core::io::OutputState;

use crate::{DoorCommand, DoorEvent};

use super::{Blocked::Blocked, DoorStateContainer, DoorStateImpl, Emergency::Emergency, NormalOperation::NormalOperation};


#[derive(Copy, Clone)]
pub struct ReleasedOnce{}

impl DoorStateImpl for ReleasedOnce
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::ValidDoorOpenRequestSeen(_ap_id) => { /* Ignore */ }
            DoorEvent::Opened => {
                    // ToDo: Start timer, that triggers a door-open-too-long alarm,
                    // if the door is not closed.                    
                    commands.push(DoorCommand::ArmDoorOpenTooLongAlarm);
                    commands.push(DoorCommand::ToggleElectricStrike(OutputState::Low));
                    commands.push(DoorCommand::DisarmAutoswitchToNormal);
                }
            DoorEvent::Closed => {
                    commands.push(DoorCommand::DisarmDoorOpenTooLongAlarm);
                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::Low));
                    return DoorStateContainer::NormalOp(NormalOperation{});
                }
            DoorEvent::DoorOpenProfileActive   => { /* ToDo -> This should propagate to NormalOp! */ }
            DoorEvent::DoorOpenProfileInactive => { /* ToDo -> This should propagate to NormalOp! */ }

            DoorEvent::BlockingContactEngaged    => {return DoorStateContainer::Blocked(Blocked{});}
            DoorEvent::BlockingContactDisengaged => {
                                            panic!("BlockingContactDisengaged in ReleasedOnce")
                                        }
            DoorEvent::ReleaseSwitchEngaged    => {return DoorStateContainer::Emergency(Emergency{});}
            DoorEvent::ReleaseSwitchDisengaged => {
                                            panic!("ReleaseSwitchDisengaged in ReleasedOnce")
                                        }

            DoorEvent::DoorOpenerKeyTriggered => { /* Ignore */ }
            DoorEvent::DoorHandleTriggered    => { /* Ignore */ }
            DoorEvent::DoorOpenTooLong => {}
            DoorEvent::DoorTimerExpired => {
                // Triggered by the pway in cases, where we have no FC.
                commands.push(DoorCommand::ToggleElectricStrike(OutputState::Low));
                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::Low));
                return DoorStateContainer::NormalOp(NormalOperation{});
            }
        }
        return DoorStateContainer::ReleasedOnce(self)
    }
}
