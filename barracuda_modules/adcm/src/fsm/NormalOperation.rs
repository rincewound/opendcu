use barracuda_core::io::OutputState;

use crate::{DoorCommand, DoorEvent};

use super::{Blocked::Blocked, DoorStateContainer, DoorStateImpl, Emergency::Emergency, ReleasedOnce::ReleasedOnce, ReleasedPermanently::ReleasedPermanently};

#[derive(Copy, Clone)]
pub struct NormalOperation{}

impl DoorStateImpl for NormalOperation
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::ValidDoorOpenRequestSeen(ap_id) => {
                                    commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                    commands.push(DoorCommand::ArmAutoswitchToNormal);   
                                    commands.push(DoorCommand::ShowSignal(ap_id, barracuda_core::sig::SigType::AccessGranted));
                                    return DoorStateContainer::ReleasedOnce(ReleasedOnce{});
                                }
            DoorEvent::Opened => {
                                    // Door forced open!
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::High))
                                }
            DoorEvent::Closed => {
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::Low))
                                }
            DoorEvent::DoorOpenProfileActive => {
                                    commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));                
                                    return DoorStateContainer::ReleasePerm(ReleasedPermanently{});
                                }
            DoorEvent::DoorOpenProfileInactive => {
                                    panic!("DoorOpenProfileInactive in NormalOperation")
                                }
            DoorEvent::BlockingContactEngaged => { return DoorStateContainer::Blocked(Blocked{}); }
            DoorEvent::BlockingContactDisengaged => {
                                    panic!("BlockingContactDisengaged in NormalOperation")
                                }
            DoorEvent::ReleaseSwitchEngaged => { return DoorStateContainer::Emergency(Emergency{}); }
            DoorEvent::ReleaseSwitchDisengaged => {
                                    panic!("ReleaseSwitchDisengaged in NormalOperation")                
                                }
            DoorEvent::DoorOpenerKeyTriggered => {
                                commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                return DoorStateContainer::ReleasedOnce(ReleasedOnce{});
            }
            DoorEvent::DoorHandleTriggered => {
                                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                return DoorStateContainer::ReleasedOnce(ReleasedOnce{});
            }
            DoorEvent::DoorOpenTooLong => {}
            DoorEvent::DoorTimerExpired => {}
        }
        return DoorStateContainer::NormalOp(self)
    }
}