use crate::{DoorEvent, components::InputComponent};
use barracuda_base_modules::io::{InputEvent, InputState};
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct DoorOpenerKey
{
    id: u32
}

impl DoorOpenerKey
{  }

impl InputComponent for DoorOpenerKey
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>) {

        if event.input_id != self.id 
        {
            return;
        }

        if event.state == InputState::Low
        {
            return;
        }

        generated_events.push(DoorEvent::DoorOpenerKeyTriggered);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dok() -> (DoorOpenerKey, Vec<DoorEvent>)
    {
        let fc = DoorOpenerKey {id: 24};
        let v = Vec::<DoorEvent>::new();
        return (fc, v);
    }

    #[test]
    fn on_input_event_will_ignore_events_with_non_matching_id()
    { 
        let (mut dok, mut v) = make_dok();
        let event = InputEvent{input_id: 13, state: InputState::High};
        dok.on_input_change(&event,  &mut v);
        assert!(v.len() == 0);
    }

    #[test]
    fn on_input_event_input_low_will_ignore_event()
    { 
        let (mut dok, mut v) = make_dok();
        let event = InputEvent{input_id: 24, state: InputState::Low};
        dok.on_input_change(&event,  &mut v);
        assert!(v.len() == 0);
    }

    #[test]
    fn on_input_event_will_trigger_release_once()
    {
        let (mut dok, mut v) = make_dok();
        let event = InputEvent{input_id: 24, state: InputState::High};
        dok.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::DoorOpenerKeyTriggered);
    }
}
