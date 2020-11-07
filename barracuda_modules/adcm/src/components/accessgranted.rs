use barracuda_core::{
    core::channel_manager::ChannelManager, profile::ProfileChangeEvent
};
use crate::DoorEvent;

use super::{
    OutputComponent, 
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

    fn on_door_command(&mut self, command: crate::DoorCommand) {
        match command
        {
            crate::DoorCommand::ToggleAccessAllowed(output_state) => {self.output_component.control_output(output_state)}
            _ => {}
        }
    }
}