use barracuda_core::{core::channel_manager::ChannelManager};

use crate::{DoorCommand, components::OutputComponent};

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

    pub fn from_setting(setting: OutputComponentSetting, chm: &mut ChannelManager ) -> Self
    {
        Self::new(setting.id, setting.operation_time, chm)
    }
}

impl OutputComponent for ElectricStrike
{
    fn on_door_command(&mut self, command: DoorCommand) 
    {
        match command
        {
            DoorCommand::ToggleElectricStrike(state) => { self.output_component.control_output(state)},
            DoorCommand::ToggleElectricStrikeTimed(state)=> {self.output_component.control_output_with_timeout(state)},
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use barracuda_core::io::{OutputState, OutputSwitch};

    use super::*;

    fn make_strike() -> (ElectricStrike, ChannelManager)
    {
        let mut chm = ChannelManager::new();
        let strike = ElectricStrike::new(32, 1000, &mut chm);
        return (strike, chm);
    }

    #[test]
    fn will_switch_on_on_switch_cmd()
    {
        let (mut strike, mut chm) = make_strike();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        strike.on_door_command(DoorCommand::ToggleElectricStrike(OutputState::High));

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();
        assert_eq!(cmd.target_state, OutputState::High);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.output_id, 32);

    }

    #[test]
    fn will_switch_off_on_switch_cmd()
    {
        let (mut strike, mut chm) = make_strike();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        strike.on_door_command(DoorCommand::ToggleElectricStrike(OutputState::Low));

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();
        assert_eq!(cmd.target_state, OutputState::Low);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.output_id, 32);

    }
}