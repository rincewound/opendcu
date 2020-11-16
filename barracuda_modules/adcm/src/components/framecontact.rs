use crate::DoorEvent;

use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct FrameContact
{
    id: u32,
    has_access_allowed: bool
}

impl InputComponent for FrameContact
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>) {

        if event.input_id != self.id 
        {
            return;
        }

        if event.state == InputState::High
        {
            generated_events.push(DoorEvent::Closed)
        }
        if event.state == InputState::Low
        {
            generated_events.push(DoorEvent::Opened)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fc() -> (FrameContact, Vec<DoorEvent>)
    {
        let fc = FrameContact {id: 24, has_access_allowed: false};
        let v = Vec::<DoorEvent>::new();
        return (fc, v);
    }

    #[test]
    fn on_input_event_will_ignore_events_with_non_matching_id()
    { 
        let (mut fc, mut v) = make_fc();
        let event = InputEvent{input_id: 13, state: InputState::Low};
        fc.on_input_change(&event,  &mut v);
        assert!(v.len() == 0);
    }

    #[test]
    fn on_input_event_will_trigger_door_closed()
    {
        let (mut fc, mut v) = make_fc();
        let event = InputEvent{input_id: 24, state: InputState::High};
        fc.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::Closed);     
    }

    #[test]
    fn on_input_event_will_trigger_door_opened()
    {
        let (mut fc, mut v) = make_fc();
        let event = InputEvent{input_id: 24, state: InputState::Low};
        fc.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::Opened);     
    }
}
