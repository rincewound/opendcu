use super::*;
use super::DoorEvent;
use super::outputcomponentbase::*;

pub struct ElectricStrike
{
    output_component: OutputComponentBase
}

impl ElectricStrike
{
    pub fn new(id: u32, operation_time: u64, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            output_component: OutputComponentBase::new(id, operation_time, chm)
        }
    }
}

impl OutputComponent for ElectricStrike
{
    fn on_profile_change(&mut self, _event: &ProfileChangeEvent, _generated_events: &mut Vec<DoorEvent>)
    {

    }

    fn on_door_event(&mut self, event: DoorEvent, _generated_events: &mut Vec<DoorEvent>)
    {
        match event
        {
            DoorEvent::Opened               => { self.output_component.control_output(OutputState::Low);}
            DoorEvent::ReleasedPermanently  => { self.output_component.control_output(OutputState::High);}
            DoorEvent::ReleaseOnce          => { self.output_component.control_output_with_timeout(OutputState::High);}
            DoorEvent::NormalOperation      => { self.output_component.control_output_with_timeout(OutputState::High);}
            _ => {}
        }
    }
}