use super::*;
use super::DoorEvent;

struct FrameContact
{
    id: u32
}

impl FrameContact
{
    fn handle_door_closed(&mut self)
    {
        self.trigger_door_event(DoorEvent::Closed);
    }

    fn handle_door_opened(&mut self)
    {
        // The semantics of an opened door depend on the doorstate,
        // but these are not handled here.
        self.trigger_door_event(DoorEvent::Opened);
    }

    fn trigger_door_event(&mut self, event: DoorEvent){}
}

impl InputComponent for FrameContact
{
    fn on_input_change(&mut self, event: &InputEvent) {
        if event.input_id == self.id
        {
            if event.state == InputState::_High
            {
                self.handle_door_closed();
            }
            if event.state == InputState::_Low
            {
                self.handle_door_opened();
            }
        }
    }

    fn on_door_event(&mut self, event: DoorEvent) { }
}