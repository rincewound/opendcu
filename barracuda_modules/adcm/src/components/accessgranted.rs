use super::*;
use super::DoorEvent;
use outputcomponentbase::OutputComponentBase;

struct AccessGranted
{
    output_component: OutputComponentBase
}

impl AccessGranted
{
    pub fn new(id: u32, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            output_component: OutputComponentBase::new(id, 0, chm)
        }
    }
}

impl OutputComponent for AccessGranted
{
    fn on_profile_change(&mut self, event: &ProfileChangeEvent)
    {

    }

    fn on_door_event(&mut self, event: DoorEvent)
    {
        match event
        {
            DoorEvent::Closed               => { self.output_component.control_output(OutputState::Low);}
            DoorEvent::ReleasedPermanently  => { self.output_component.control_output(OutputState::High);}
            DoorEvent::ReleaseOnce          => { self.output_component.control_output(OutputState::High);}
            DoorEvent::NormalOperation      => { self.output_component.control_output(OutputState::Low);}
            DoorEvent::ForcedOpen           => { self.output_component.control_output(OutputState::Low);} // should never happen!
            DoorEvent::DoorOpenAlarm        => { self.output_component.control_output(OutputState::Low);}
            _ => {}
        }
    }
}