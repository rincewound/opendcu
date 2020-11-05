use super::*;
use super::DoorEvent;
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

    fn on_door_event(&mut self, event: DoorEvent, _generated_events: &mut Vec<DoorEvent>)
    {
        match event
        {
            DoorEvent::ForcedOpen  => { self.output_component.control_output(OutputState::High);}
            DoorEvent::Closed      => { self.output_component.control_output(OutputState::Low);}
            _ => {return;}
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn make_alarm_relay() -> (AlarmRelay, ChannelManager, Vec::<DoorEvent>)
    {
        let mut chm = ChannelManager::new();
        let strike = AlarmRelay::new(32, &mut chm);
        let v = Vec::<DoorEvent>::new();
        return (strike, chm, v);
    }

    #[test]
    fn will_fire_on_door_forced_open()
    {
        let (mut alarm, mut chm, mut events) = make_alarm_relay();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        alarm.on_door_event(DoorEvent::ForcedOpen, &mut events);

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();

        assert_eq!(cmd.output_id, 32);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.target_state, OutputState::High);
    }

    #[test]
    fn will_switch_off_on_door_closed_open()
    {
        let (mut alarm, mut chm, mut events) = make_alarm_relay();
        let output_cmds = chm.get_receiver::<OutputSwitch>();
        alarm.on_door_event(DoorEvent::Closed, &mut events);

        assert!(output_cmds.has_data());
        let cmd = output_cmds.receive();

        assert_eq!(cmd.output_id, 32);
        assert_eq!(cmd.switch_time, 0);
        assert_eq!(cmd.target_state, OutputState::Low);
    }
}