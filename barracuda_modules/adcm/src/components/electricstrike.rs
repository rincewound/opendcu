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

    pub fn from_setting(setting: OutputComponentSetting, chm: &mut ChannelManager ) -> Self
    {
        Self::new(setting.id, setting.operation_time, chm)
    }
}

impl OutputComponent for ElectricStrike
{
    fn on_profile_change(&mut self, _event: &ProfileChangeEvent, _generated_events: &mut Vec<DoorEvent>)
    {    }

    fn on_door_event(&mut self, event: DoorEvent, _generated_events: &mut Vec<DoorEvent>)
    {
        match event
        {
            DoorEvent::Opened               => { self.output_component.control_output(OutputState::Low);}
            DoorEvent::ReleasedPermanently  => { self.output_component.control_output(OutputState::High);}
            DoorEvent::ReleaseOnce          => { self.output_component.control_output_with_timeout(OutputState::High);}
            DoorEvent::NormalOperation      => { self.output_component.control_output(OutputState::Low);} 
            DoorEvent::EmergencyRelease     => { self.output_component.control_output(OutputState::High);}
            _ => {}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn make_strike() -> (ElectricStrike, ChannelManager, Vec::<DoorEvent>)
    {
        let mut chm = ChannelManager::new();
        let strike = ElectricStrike::new(32, 1000, &mut chm);
        let v = Vec::<DoorEvent>::new();
        return (strike, chm, v);
    }

    #[test]
    fn will_switch_off_when_door_is_opened()
    {
        let (mut strike, mut chm, mut events) = make_strike();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        strike.on_door_event(DoorEvent::Opened, &mut events);

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();

        assert_eq!(cmd.output_id, 32);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.target_state, OutputState::Low);

    }

    #[test]
    fn will_switch_on_timed_on_release_once()
    {
        let (mut strike, mut chm, mut events) = make_strike();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        strike.on_door_event(DoorEvent::ReleaseOnce, &mut events);

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();
        assert_eq!(cmd.target_state, OutputState::High);
        assert_eq!(cmd.switch_time, 1000);
        assert_eq!(cmd.output_id, 32);

    }

    #[test]
    fn will_switch_on_on_release_permanently()
    {
        let (mut strike, mut chm, mut events) = make_strike();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        strike.on_door_event(DoorEvent::ReleasedPermanently, &mut events);

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();
        assert_eq!(cmd.target_state, OutputState::High);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.output_id, 32);

    }

    #[test]
    fn will_switch_off_on_normal_operation()
    {
        let (mut strike, mut chm, mut events) = make_strike();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        strike.on_door_event(DoorEvent::NormalOperation, &mut events);

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();
        assert_eq!(cmd.target_state, OutputState::Low);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.output_id, 32);

    }
}