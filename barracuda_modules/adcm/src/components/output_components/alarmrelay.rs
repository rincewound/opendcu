use barracuda_core::{core::channel_manager::ChannelManager, profile::ProfileChangeEvent};

use crate::{DoorCommand, DoorEvent, components::OutputComponent};

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
    fn on_profile_change(&mut self, _event: &ProfileChangeEvent, _generated_events: &mut Vec<DoorEvent>)
    {    }

    fn on_door_command(&mut self, command: DoorCommand) {
        match command
        {
            DoorCommand::ToggleAlarmRelay(output_state) => {self.output_component.control_output(output_state)}
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use barracuda_core::io::{OutputState, OutputSwitch};

    use super::*;

    fn make_alarm_relay() -> (AlarmRelay, ChannelManager, Vec::<DoorEvent>)
    {
        let mut chm = ChannelManager::new();
        let strike = AlarmRelay::new(32, &mut chm);
        let v = Vec::<DoorEvent>::new();
        return (strike, chm, v);
    }

    #[test]
    fn will_fire_on_cmd()
    {
        let (mut alarm, mut chm, _events) = make_alarm_relay();
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
        let (mut alarm, mut chm, _events) = make_alarm_relay();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        alarm.on_door_command(DoorCommand::ToggleAlarmRelay(OutputState::Low));

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();

        assert_eq!(cmd.output_id, 32);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.target_state, OutputState::Low);
    }
}