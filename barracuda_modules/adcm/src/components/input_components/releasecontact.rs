use crate::{DoorEvent, components::InputComponent};
use barracuda_base_modules::io::{InputEvent, InputState};
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize, Clone, Copy)]
pub struct ReleaseContact
{
    id: u32
}

impl ReleaseContact
{  }

impl InputComponent for ReleaseContact
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>) {

        if event.input_id != self.id 
        {
            return;
        }

        if event.state == InputState::Low
        {
            generated_events.push(DoorEvent::ReleaseSwitchDisengaged);
        }
        else if event.state == InputState::High
        {
            generated_events.push(DoorEvent::ReleaseSwitchEngaged);
        }        
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rel() -> (ReleaseContact, Vec<DoorEvent>)
    {
        let fc = ReleaseContact {id: 24};
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
        assert!(v[0] == DoorEvent::ReleaseSwitchDisengaged);
    }

    #[test]
    fn on_input_event_will_trigger_permanent_release()
    {
        let (mut dok, mut v) = make_rel();
        let event = InputEvent{input_id: 24, state: InputState::High};
        dok.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::ReleaseSwitchEngaged);
    }
}
