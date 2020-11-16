use crate::{DoorCommand, DoorEvent};

pub mod NormalOperation;
mod ReleasedOnce;
mod ReleasedPermanently;
mod Blocked;
mod Emergency;


#[derive(Copy, Clone)]
pub enum DoorStateContainer
{
    NormalOp(NormalOperation::NormalOperation),
    ReleasedOnce(ReleasedOnce::ReleasedOnce),
    ReleasePerm(ReleasedPermanently::ReleasedPermanently),
    Blocked(Blocked::Blocked),
    Emergency(Emergency::Emergency)
}


// Use Enum dispatch here!
pub trait DoorStateImpl
{
    fn dispatch_door_event(self, d: DoorEvent, commands: &mut Vec<DoorCommand>) -> DoorStateContainer;
}

#[cfg(test)]
mod normal_op_tests 
{    
    use super::*;
    use super::NormalOperation::NormalOperation;
    use super::ReleasedOnce::ReleasedOnce;
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
        assert_eq!(v.len() , 3);
        assert_eq!(v[0], DoorCommand::ToggleElectricStrikeTimed(OutputState::High));
        assert_eq!(v[1], DoorCommand::ToggleAccessAllowed(OutputState::High));
        assert_eq!(v[2], DoorCommand::ArmAutoswitchToNormal);
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

#[cfg(test)]
mod released_once_tests 
{
    use super::*;
    use super::ReleasedOnce::ReleasedOnce;
    use super::NormalOperation::NormalOperation;
    use barracuda_core::io::OutputState;

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
        assert_eq!(3, v.len());
        assert_eq!(v[0], DoorCommand::ArmDoorOpenTooLongAlarm);
        assert_eq!(v[1], DoorCommand::ToggleElectricStrike(OutputState::Low));
        assert_eq!(v[2], DoorCommand::DisarmAutoswitchToNormal);
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
