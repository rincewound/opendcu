use barracuda_core::{core::channel_manager::ChannelManager};

use crate::{DoorCommand, components::OutputComponent};

use super::outputcomponentbase::*;

pub struct AlarmRelay
{
    output_component: OutputComponentBase
}

impl AlarmRelay
{
    pub fn new(id: u32, chm: &mut ChannelManager) -> Self
    {
        Self
        {
            output_component: OutputComponentBase::new(id, 0, chm)
        }
    }
}

impl OutputComponent for AlarmRelay
{
    fn on_door_command(&mut self, command: DoorCommand) 
    {
        match command
        {
            DoorCommand::ToggleAlarmRelay(output_state) => {self.output_component.control_output(output_state)}
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {

    use barracuda_base_modules::io::{OutputState, OutputSwitch};

    use super::*;

    fn make_alarm_relay() -> (AlarmRelay, ChannelManager)
    {
        let mut chm = ChannelManager::new();
        let strike = AlarmRelay::new(32, &mut chm);
        return (strike, chm);
    }

    #[test]
    fn will_fire_on_cmd()
    {
        let (mut alarm, mut chm) = make_alarm_relay();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        alarm.on_door_command(DoorCommand::ToggleAlarmRelay(OutputState::High));

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();

        assert_eq!(cmd.output_id, 32);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.target_state, OutputState::High);
    }

    #[test]
    fn will_switch_off_on_cmd()
    {
        let (mut alarm, mut chm) = make_alarm_relay();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        alarm.on_door_command(DoorCommand::ToggleAlarmRelay(OutputState::Low));

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();

        assert_eq!(cmd.output_id, 32);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.target_state, OutputState::Low);
    }
}