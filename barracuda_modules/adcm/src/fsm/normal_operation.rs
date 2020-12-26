

use barracuda_base_modules::{events::LogEvent, io::OutputState};
use barracuda_base_modules::sig::SigType;

use crate::{DoorCommand, DoorEvent};

use super::{DoorStateContainer, DoorStateImpl, released_once::ReleasedOnce, released_permanently::ReleasedPermanently, emergency::Emergency, blocked::Blocked};

#[derive(Copy, Clone)]
pub struct NormalOperation{}

impl DoorStateImpl for NormalOperation
{
    fn dispatch_door_event(self,passageway_id: u32, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::ValidDoorOpenRequestSeen(ap_id, token) => {
                                    commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                    commands.push(DoorCommand::ArmAutoswitchToNormal);   
                                    commands.push(DoorCommand::ShowSignal(ap_id, SigType::AccessGranted));
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::AccessGranted(passageway_id, token, ap_id)));
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::DoorReleasedOnce(passageway_id)));
                                    return DoorStateContainer::ReleasedOnce(ReleasedOnce{}, passageway_id);
                                }
            DoorEvent::Opened => {
                                    // Door forced open!
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::High));
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::DoorForcedOpen(passageway_id)));
                                }
            DoorEvent::Closed => {
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::Low));
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::DoorClosedAgain(passageway_id))); 
                                }
            DoorEvent::DoorOpenProfileActive => {
                                    commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));      
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::DoorPermantlyReleased(passageway_id)));
                                    return DoorStateContainer::ReleasePerm(ReleasedPermanently{}, passageway_id);
                                }            

            DoorEvent::DoorOpenerKeyTriggered => {
                                commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorReleasedOnce(passageway_id)));
                                return DoorStateContainer::ReleasedOnce(ReleasedOnce{}, passageway_id);
            }
            DoorEvent::DoorHandleTriggered => {
                                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorReleasedOnce(passageway_id)));
                                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                return DoorStateContainer::ReleasedOnce(ReleasedOnce{}, passageway_id);
            }

            DoorEvent::BlockingContactEngaged => { 
                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorBlocked(passageway_id)));
                return DoorStateContainer::Blocked(Blocked{}, passageway_id); 
            }            
            DoorEvent::ReleaseSwitchEngaged => { 
                commands.push(DoorCommand::TriggerEvent(LogEvent::DoorEmergencyReleased(passageway_id)));
                return DoorStateContainer::Emergency(Emergency{}, passageway_id); 
            }
            
            DoorEvent::DoorOpenProfileInactive => {}
            DoorEvent::DoorOpenTooLong => {}
            DoorEvent::DoorTimerExpired => {}
            DoorEvent::BlockingContactDisengaged => { }
            DoorEvent::ReleaseSwitchDisengaged => { }
        }
        return DoorStateContainer::NormalOp(self, passageway_id)
    }
}


#[cfg(test)]
mod normal_op_tests 
{    
    use super::*;

    fn make_normal_op() -> (NormalOperation, Vec<DoorCommand>)
    {
        let op = NormalOperation{};
        let v =  Vec::<DoorCommand>::new();
        return(op, v);
    }

    fn assert_states_are_equal(s1: DoorStateContainer, s2: DoorStateContainer)
    {
        assert_eq!(std::mem::discriminant(&s1), std::mem::discriminant(&s2));
    }

    #[test]
    pub fn normal_op_generates_release_cmd_on_valid_booking()
    {
        let (op, mut v) = make_normal_op();
        op.dispatch_door_event(DoorEvent::ValidDoorOpenRequestSeen(0), &mut v);
        assert_eq!(v.len() , 4);
        assert_eq!(v[0], DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
        assert_eq!(v[1], DoorCommand::ToggleAccessAllowed(OutputState::High));
        assert_eq!(v[2], DoorCommand::ArmAutoswitchToNormal);
        assert_eq!(v[3], DoorCommand::ShowSignal(0, SigType::AccessGranted));
    }

    #[test]
    pub fn normal_op_changes_to_released_once_on_valid_booking()
    {
        let (op, mut v) = make_normal_op();
        let next = op.dispatch_door_event(DoorEvent::ValidDoorOpenRequestSeen(0), &mut v);
        assert_states_are_equal(next, DoorStateContainer::ReleasedOnce(ReleasedOnce{}))
    }

    #[test]
    pub fn normal_op_generates_release_cmd_on_door_opener_key()
    {
        let (op, mut v) = make_normal_op();
        op.dispatch_door_event(DoorEvent::DoorOpenerKeyTriggered, &mut v);
        assert_eq!(v.len() , 2);
        assert_eq!(v[0], DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
        assert_eq!(v[1], DoorCommand::ToggleAccessAllowed(OutputState::High));
    }

    #[test]
    pub fn normal_op_changes_to_released_once_on_door_opener_key()
    {
        let (op, mut v) = make_normal_op();
        let next = op.dispatch_door_event(DoorEvent::DoorOpenerKeyTriggered, &mut v);
        assert_states_are_equal(next, DoorStateContainer::ReleasedOnce(ReleasedOnce{}))
    }

    #[test]
    pub fn normal_op_generates_access_allowed_on_door_handle()
    {
        let (op, mut v) = make_normal_op();
        op.dispatch_door_event(DoorEvent::DoorHandleTriggered, &mut v);
        assert_eq!(v.len() , 1);
        assert_eq!(v[0], DoorCommand::ToggleAccessAllowed(OutputState::High));
    }

    #[test]
    pub fn normal_op_changes_to_released_once_on_door_handle()
    {
        let (op, mut v) = make_normal_op();
        let next = op.dispatch_door_event(DoorEvent::DoorHandleTriggered, &mut v);
        assert_states_are_equal(next, DoorStateContainer::ReleasedOnce(ReleasedOnce{}))
    }

    #[test]
    pub fn normal_op_fires_alarm_on_door_open()
    {
        let (op, mut v) = make_normal_op();
        op.dispatch_door_event(DoorEvent::Opened, &mut v);
        assert_eq!(v[0], DoorCommand::ToggleAlarmRelay(OutputState::High));
    }

    #[test]
    pub fn normal_op_disengages_alarm_on_door_open()
    {
        let (op, mut v) = make_normal_op();
        op.dispatch_door_event(DoorEvent::Closed, &mut v);
        assert_eq!(v[0], DoorCommand::ToggleAlarmRelay(OutputState::Low));
    }

}