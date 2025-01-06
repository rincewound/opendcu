use barracuda_base_modules::events::LogEvent;

use super::{normal_operation::NormalOperation, DoorStateContainer, DoorStateImpl};
use crate::{DoorCommand, DoorEvent};

#[derive(Copy, Clone)]
pub struct Emergency {}

impl DoorStateImpl for Emergency {
    fn dispatch_door_event(
        self,
        passageway_id: u32,
        d: DoorEvent,
        commands: &mut Vec<DoorCommand>,
    ) -> DoorStateContainer {
        match d {
            DoorEvent::ReleaseSwitchDisengaged => {
                commands.push(DoorCommand::TriggerEvent(
                    LogEvent::DoorEnteredNormalOperation(passageway_id),
                ));
                return DoorStateContainer::NormalOp(NormalOperation {}, passageway_id);
            }
            _ => {}
        }
        return DoorStateContainer::Emergency(self, passageway_id);
    }
}
