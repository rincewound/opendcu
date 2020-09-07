use super::*;
use super::DoorEvent;

use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
pub struct FrameContact
{
    id: u32,
    has_access_allowed: bool
}

impl FrameContact
{
    fn handle_door_closed(&mut self, generated_events: &mut Vec<DoorEvent>)
    {
        self.has_access_allowed = false;
        self.trigger_door_event(DoorEvent::Closed, generated_events);
    }

    fn handle_door_opened(&mut self, generated_events: &mut Vec<DoorEvent>)
    {
        // The semantics of an opened door depend on the doorstate,
        // but these are not handled here.
        if self.has_access_allowed
        {
            self.trigger_door_event(DoorEvent::Opened, generated_events);
        }
        else 
        {
            self.trigger_door_event(DoorEvent::ForcedOpen, generated_events);
        }
    }

    fn trigger_door_event(&mut self, event: DoorEvent, generated_events: &mut Vec<DoorEvent>)
    {
        generated_events.push(event);
    }
}

impl InputComponent for FrameContact
{
    fn on_input_change(&mut self, event: &InputEvent, generated_events: &mut Vec<DoorEvent>) {

        if event.input_id != self.id 
        {
            return;
        }

        if event.state == InputState::_High
        {
            self.handle_door_closed(generated_events);
        }
        if event.state == InputState::_Low
        {
            self.handle_door_opened(generated_events);
        }
    }

    fn on_door_event(&mut self, event: DoorEvent, _generated_events: &mut Vec<DoorEvent>) 
    { 
        match event
        {
            DoorEvent::Opened => {}
            DoorEvent::Closed => {}
            DoorEvent::ForcedOpen => {}
            DoorEvent::_OpenTooLong => {}
            DoorEvent::DoorOpenAlarm => {}
            DoorEvent::ReleasedPermanently => {}
            DoorEvent::ReleaseOnce => {self.has_access_allowed = true}
            DoorEvent::NormalOperation => {}
            DoorEvent::_Block => {}
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
        let event = InputEvent{input_id: 13, state: InputState::_Low};
        fc.on_input_change(&event,  &mut v);
        assert!(v.len() == 0);
    }

    #[test]
    fn on_input_event_will_trigger_door_forced_open_if_no_access_granted_was_sent_before()
    {
        let (mut fc, mut v) = make_fc();
        let event = InputEvent{input_id: 24, state: InputState::_Low};
        fc.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::ForcedOpen);
    }

    #[test]
    fn on_input_event_will_trigger_door_forced_open_after_door_close()
    {
        let (mut fc, mut v) = make_fc();
        let event = InputEvent{input_id: 24, state: InputState::_Low};

        // Normal sequence: ReleaseOnce, OpenDoor, CloseDoor
        fc.on_door_event(DoorEvent::ReleaseOnce, &mut v);
        fc.on_input_change(&event,  &mut v);
        let event2 = InputEvent{input_id: 24, state: InputState::_High};
        fc.on_input_change(&event2,  &mut v);

        // If we reopen the door now, we should see a forced open:

        v.clear();  // we don't care for the previous events anymore

        fc.on_input_change(&event, &mut v);
        assert!(v[0] == DoorEvent::ForcedOpen);
    }

    #[test]
    fn on_input_event_will_trigger_door_opened_if_access_granted_was_sent_before()
    {
        let (mut fc, mut v) = make_fc();
        let event = InputEvent{input_id: 24, state: InputState::_Low};
        fc.on_door_event(DoorEvent::ReleaseOnce, &mut v);
        fc.on_input_change(&event,  &mut v);
        assert!(v[0] == DoorEvent::Opened);     
    }

    // ToDo: Tests for permanent release!
    //       Tests for DoorOpenTooLong (-> should these alarms be handled elsewhere to keep FC simple?)
}
