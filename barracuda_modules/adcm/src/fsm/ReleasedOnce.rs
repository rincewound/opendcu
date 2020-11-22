use barracuda_core::io::OutputState;

use crate::{DoorCommand, DoorEvent};

use super::{Blocked::Blocked, DoorStateContainer, DoorStateImpl, Emergency::Emergency, NormalOperation::NormalOperation, ReleasedPermanently::ReleasedPermanently};


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
            DoorEvent::DoorOpenProfileActive => { 
                    commands.push(DoorCommand::ToggleElectricStrike(OutputState::High));
                    commands.push(DoorCommand::ToggleAccessAllowed(OutputState::High));
                    return DoorStateContainer::ReleasePerm(ReleasedPermanently{})
                }            
            
            DoorEvent::DoorTimerExpired => {
                // Triggered by the pway in cases, where we have no FC.
                commands.push(DoorCommand::ToggleElectricStrike(OutputState::Low));
                commands.push(DoorCommand::ToggleAccessAllowed(OutputState::Low));
                return DoorStateContainer::NormalOp(NormalOperation{});
            }
            DoorEvent::BlockingContactEngaged => {return DoorStateContainer::Blocked(Blocked{});}
            DoorEvent::ReleaseSwitchEngaged    => {return DoorStateContainer::Emergency(Emergency{});}

            DoorEvent::BlockingContactDisengaged => {/* Ignore */ }
            DoorEvent::ReleaseSwitchDisengaged => { /* Ignore */ }
            DoorEvent::DoorOpenProfileInactive => { /* Ignore */ }
            DoorEvent::DoorOpenerKeyTriggered => { /* Ignore */ }
            DoorEvent::DoorHandleTriggered    => { /* Ignore */ }
            DoorEvent::DoorOpenTooLong        => { /* Ignore */ }
        }
        return DoorStateContainer::ReleasedOnce(self)
    }
}


#[cfg(test)]
mod released_once_tests 
{
    use super::*;
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
