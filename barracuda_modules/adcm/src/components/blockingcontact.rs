use super::*;
use super::DoorEvent;

use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct BlockingContact
{
    id: u32
}

impl BlockingContact
{  }

impl InputComponent for BlockingContact
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>) {

        if event.input_id != self.id 
        {
            return;
        }

        if event.state == InputState::Low
        {
            generated_events.push(DoorEvent::NormalOperation);
        }
        else if event.state == InputState::High
        {
            generated_events.push(DoorEvent::Block);
        }        
    }

    fn on_door_event(&mut self, _event: DoorEvent, _generated_events: &mut Vec<DoorEvent>) { }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rel() -> (BlockingContact, Vec<DoorEvent>)
    {
        let fc = BlockingContact {id: 24};
        let v = Vec::<DoorEvent>::new();
        return (fc, v);
    }

    #[test]
    fn on_input_event_will_ignore_events_with_non_matching_id()
    { 
        let (mut doh, mut v) = make_rel();
        let event = InputEvent{input_id: 13, state: InputState::High};
        doh.on_input_change(&event,  &mut v);
        assert!(v.len() == 0);
    }

    #[test]
    fn on_input_event_input_low_will_generates_normal_operation()
    { 
        let (mut dok, mut v) = make_rel();
        let event = InputEvent{input_id: 24, state: InputState::Low};
        dok.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::NormalOperation);
    }

    #[test]
    fn on_input_event_will_trigger_block()
    {
        let (mut dok, mut v) = make_rel();
        let event = InputEvent{input_id: 24, state: InputState::High};
        dok.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::Block);
    }
}
