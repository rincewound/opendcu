use barracuda_core::{
    core::channel_manager::ChannelManager, io::OutputState, profile::ProfileChangeEvent
};
use super::{
    DoorEvent, OutputComponent, 
    outputcomponentbase::{OutputComponentBase, OutputComponentSetting}
};

pub struct AccessGranted
{
    output_component: OutputComponentBase
}

impl AccessGranted
{
    pub fn new(id: u32, operation_time: u64, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            output_component: OutputComponentBase::new(id, operation_time, chm)
        }
    }
    pub fn from_setting(setting: OutputComponentSetting, chm: &mut ChannelManager ) -> Self
    {
        Self::new(setting.id, setting.operation_time, chm)
    }
}

impl OutputComponent for AccessGranted
{
    fn on_profile_change(&mut self, _event: &ProfileChangeEvent, _generated_events: &mut Vec<DoorEvent>)
    {

    }

    fn on_door_event(&mut self, event: DoorEvent, _generated_events: &mut Vec<DoorEvent>)
    {
        match event
        {
            DoorEvent::Closed               => { self.output_component.control_output(OutputState::Low);}
            // ToDo: Check if AccessAllows suffices here
            DoorEvent::AccessAllowed        => { self.output_component.control_output(OutputState::High);}
            DoorEvent::ReleasedPermanently  => { self.output_component.control_output(OutputState::High);}
            DoorEvent::ReleaseOnce          => { self.output_component.control_output(OutputState::High);}
            DoorEvent::NormalOperation      => { self.output_component.control_output(OutputState::Low);}
            DoorEvent::ForcedOpen           => { self.output_component.control_output(OutputState::Low);} // should never happen!
            DoorEvent::DoorOpenAlarm        => { self.output_component.control_output(OutputState::Low);}
            _ => {}
        }
    }
}