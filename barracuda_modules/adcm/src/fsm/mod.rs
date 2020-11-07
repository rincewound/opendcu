use barracuda_core::io::OutputState;

use crate::{DoorCommand, DoorEvent};


pub enum DoorStateContainer
{
    NormalOp(NormalOperation),
    ReleasedOnce(ReleasedOnce),
    //ReleasePerm,
    Blocked,
    Emergency
}


// Use Enum dispatch here!
pub trait DoorStateImpl
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer;
}

#[derive(Copy, Clone)]
pub struct NormalOperation{}

impl DoorStateImpl for NormalOperation
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::ValidDoorOpenRequestSeen => {
                                    // ToDo: Start timer to switch back to normal op, in case the door was not opened.
                                    commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                    return DoorStateContainer::ReleasedOnce(ReleasedOnce{});
                                }
            DoorEvent::Opened => {
                                    // Door forced open!
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::High))
                                }
            DoorEvent::Closed => {
                                    commands.push(DoorCommand::ToggleAlarmRelay(OutputState::Low))
                                }
            DoorEvent::DoorOpenProfileActive => {}
            DoorEvent::DoorOpenProfileInactive => {}
            DoorEvent::BlockingContactEngaged => {return DoorStateContainer::Blocked;}
            DoorEvent::BlockingContactDisengaged => {}
            DoorEvent::ReleaseSwitchEngaged => {return DoorStateContainer::Emergency;}
            DoorEvent::ReleaseSwitchDisengaged => {}
            DoorEvent::DoorOpenerKeyTriggered => {
                                commands.push(DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
                                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                return DoorStateContainer::ReleasedOnce(ReleasedOnce{});
            }
            DoorEvent::DoorHandleTriggered => {
                                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                                return DoorStateContainer::ReleasedOnce(ReleasedOnce{});
            }
        }
        return DoorStateContainer::NormalOp(self)
    }
}


#[derive(Copy, Clone)]
pub struct ReleasedOnce{}

impl DoorStateImpl for ReleasedOnce
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer {
        match d
        {
            DoorEvent::ValidDoorOpenRequestSeen => {}
            DoorEvent::Opened => {
                    // ToDo: Start timer, that triggers a door-open-too-long alarm,
                    // iff the door is not closed.
                    commands.push(DoorCommand::ArmDoorOpenTooLongAlarm);
                    commands.push(DoorCommand::ToggleElectricStrike(OutputState::Low));
                }
            DoorEvent::Closed => {
                    commands.push(DoorCommand::DisarmDoorOpenTooLongAlarm);
                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::Low));
                    return DoorStateContainer::NormalOp(NormalOperation{});
                }
            DoorEvent::DoorOpenProfileActive => {}
            DoorEvent::DoorOpenProfileInactive => {}
            DoorEvent::BlockingContactEngaged => {return DoorStateContainer::Blocked;}
            DoorEvent::BlockingContactDisengaged => {}
            DoorEvent::ReleaseSwitchEngaged => {return DoorStateContainer::Emergency;}
            DoorEvent::ReleaseSwitchDisengaged => {}
            DoorEvent::DoorOpenerKeyTriggered => {}
            DoorEvent::DoorHandleTriggered => {}
        }
        return DoorStateContainer::ReleasedOnce(self)
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
        op.dispatch_door_event(DoorEvent::ValidDoorOpenRequestSeen, &mut v);
        assert_eq!(v.len() , 2);
        assert_eq!(v[0], DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
        assert_eq!(v[1], DoorCommand::ToggleAccessAllowed(OutputState::High));
    }

    #[test]
    pub fn normal_op_changes_to_released_once_on_valid_booking()
    {
        let (op, mut v) = make_normal_op();
        let next = op.dispatch_door_event(DoorEvent::ValidDoorOpenRequestSeen, &mut v);
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

#[cfg(test)]
mod released_once_tests 
{
    use super::*;

    fn make_released_once() -> (ReleasedOnce, Vec<DoorCommand>)
    {
        let op = ReleasedOnce{};
        let v =  Vec::<DoorCommand>::new();
        return(op, v);
    }

    fn assert_states_are_equal(s1: DoorStateContainer, s2: DoorStateContainer)
    {
        assert_eq!(std::mem::discriminant(&s1), std::mem::discriminant(&s2));
    }

    #[test]
    pub fn arms_door_open_too_long_and_switches_off_strike_on_door_open()
    {
        let (op, mut v)  = make_released_once();
        op.dispatch_door_event(DoorEvent::Opened, &mut v);
        assert_eq!(2, v.len());
        assert_eq!(v[0], DoorCommand::ArmDoorOpenTooLongAlarm);
        assert_eq!(v[1], DoorCommand::ToggleElectricStrike(OutputState::Low));
    }

    
    #[test]
    pub fn disarms_door_open_too_long_and_switches_off_access_allowed_on_door_close()
    {
        let (op, mut v)  = make_released_once();
        op.dispatch_door_event(DoorEvent::Closed, &mut v);
        assert_eq!(2, v.len());
        assert_eq!(v[0], DoorCommand::DisarmDoorOpenTooLongAlarm);
        assert_eq!(v[1], DoorCommand::ToggleAccessAllowed(OutputState::Low));
    }

    #[test]
    pub fn returns_to_normal_op_on_door_close()
    {
        let (op, mut v)  = make_released_once();
        let next = op.dispatch_door_event(DoorEvent::Closed, &mut v);
        assert_states_are_equal(next, DoorStateContainer::NormalOp(NormalOperation{}))
    }
}