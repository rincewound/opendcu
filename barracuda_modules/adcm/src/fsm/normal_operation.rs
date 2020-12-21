use barracuda_core::{events::LogEvent, io::OutputState};

use crate::{DoorCommand, DoorEvent};

use super::{DoorStateContainer, DoorStateImpl, released_once::ReleasedOnce, released_permanently::ReleasedPermanently, emergency::Emergency, blocked::Blocked};

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
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::High));
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::DoorForcedOpen(1048))); // ToDo: Doorstate needs to know the door id
                                }
            DoorEvent::Closed => {
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::Low));
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::DoorClosedAgain(1048))); // ToDo: Doorstate needs to know the door id
                                }
            DoorEvent::DoorOpenProfileActive => {
                                    commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));      
                                    commands.push(DoorCommand::TriggerEvent(LogEvent::DoorPermantlyReleased(1048))); // ToDo: Doorstate needs to know the door id          
                                    return DoorStateContainer::ReleasePerm(ReleasedPermanently{});
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

            DoorEvent::BlockingContactEngaged => { return DoorStateContainer::Blocked(Blocked{}); }            
            DoorEvent::ReleaseSwitchEngaged => { return DoorStateContainer::Emergency(Emergency{}); }
            
            DoorEvent::DoorOpenProfileInactive => {}
            DoorEvent::DoorOpenTooLong => {}
            DoorEvent::DoorTimerExpired => {}
            DoorEvent::BlockingContactDisengaged => { }
            DoorEvent::ReleaseSwitchDisengaged => { }
        }
        return DoorStateContainer::NormalOp(self)
    }
}


#[cfg(test)]
mod normal_op_tests 
{    
    use super::*;
    use barracuda_core::io::OutputState;

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
        assert_eq!(v[3], DoorCommand::ShowSignal(0, barracuda_core::sig::SigType::AccessGranted));
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