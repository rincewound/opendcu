use barracuda_core::{core::{channel_manager::ChannelManager, broadcast_channel::GenericSender}, io::{OutputState, OutputSwitch}};
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
pub struct OutputComponentSetting
{
    id: u32,
    operation_time: u64
}

pub struct OutputComponentBase
{
    id: u32,
    operation_time: u64,
    output_command_tx: GenericSender<OutputSwitch>,
}

impl OutputComponentBase
{
    pub fn new(id: u32, operation_time: u64, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            id,
            operation_time,
            output_command_tx: chm.get_sender()
        }
    }

    pub fn control_output_with_timeout(&self, switch_state: OutputState)
    {
        let cmd = OutputSwitch {output_id: self.id, switch_time: self.operation_time, target_state: switch_state};
        self.output_command_tx.send(cmd);
    }

    pub fn control_output(&self, switch_state: OutputState)
    {
        let cmd = OutputSwitch {output_id: self.id, switch_time: 0, target_state: switch_state};
        self.output_command_tx.send(cmd);
    }
}