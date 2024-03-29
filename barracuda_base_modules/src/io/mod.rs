
use barracuda_core::{core::{SystemMessage, bootstage_helper::{self}, broadcast_channel::{GenericReceiver, GenericSender}, channel_manager::ChannelManager, event::DataEvent, shareable::Shareable, timer::Timer}, trace::trace_helper};
use std::{sync::Arc, thread};

use crate::modcaps::*;


extern crate chrono;

#[derive(Clone, PartialEq)]
pub enum InputState
{
    _Unknown,
    Low,
    High,
    _Short,
    _Cutout
}

// The interface of Input providing modules towards
// the IO module. All changes are propagated this way

#[derive(Clone)]
pub struct RawInputEvent
{
    input_id: u32,      // SUD!
    state: InputState
}


// Interface of the IO Module to the rest of the
// system. Logical Input states, which have been
// debounce appropiately are propagated this way
#[derive(Clone)]
pub struct InputEvent
{
    pub input_id: u32,      // Logical!
    pub state: InputState
}

/// # InputSetting
/// This struct describes a runtimesetting for
/// a given digital input. The fiels have the
/// following semantics:
/// * input_id: Contains the logical id of the
///             input, which was derived from the
///             SUD.
/// * inverted_polarity: Controls if the input is
///             considered to be active low. If
///             set to true, a physical state of
///             "Low" will be inverted to "High"
///             and vice versa
/// * The debouncetimes (in ms) control how long a given signal must not change, before an InputEvent is triggered.
// pub struct InputSetting
// {
//     input_id: u32,              //Logical!
//     inverted_polarity: bool,
//     debounce_on: u64,
//     debounce_off: u64
// }

#[derive(Copy,Clone, PartialEq, Debug)]
pub enum OutputState
{
    Low,
    High
}

// pub struct OutputSetting
// {
//     output_id: u32, // Logical!
//     inverted_polarity: bool
// }

#[derive(Clone)]
pub struct RawOutputSwitch
{
    output_id: u32,     // SUD!
    target_state: OutputState   // physical!
}

#[derive(Clone)]
pub struct OutputSwitch
{
    pub output_id: u32,
    pub target_state: OutputState,   //logical!
    pub switch_time: u64            // in ms!
}


const MODULE_ID: u32 = 0x07000000;

pub fn launch(chm: &mut ChannelManager)
{
    let tracer = trace_helper::TraceHelper::new("IO/IoManager".to_string(), chm);
    let mut ioman = IoManager::new(tracer, chm);
    thread::spawn(move || {
        ioman.init();
        loop
        {
            if !ioman.run()
            {
                break;
            }
        }

    });
}

struct OutputEntry
{
    timer_guard: Option<Arc<bool>>
}

/// # The IO Manager
/// The IO Manager is responsible for providing
/// a uniform list of input- and output ids from
/// the I/Os of a ll loaded modules.
///
/// It will map all inputs from all modules to the
/// Range [0...NumInputs]. This mapping is necessary,
/// because the internal hardware ID of an input (and output!)
/// depends on the actual module ID that hosts the
/// peripheral in question.
///
/// The IO manager will only make modules available, that
/// advertise their IOs during LLI using a
/// ModuleCapabilityAdvertisement
///
/// Note that this module only provides a generic interface
/// and does not have logic for e.g. debouncing inputs.
pub struct IoManager
{
    system_events_rx: GenericReceiver<SystemMessage>,
    system_events_tx: GenericSender<SystemMessage>,
    modcaps_rx: GenericReceiver<crate::modcaps::ModuleCapabilityAdvertisement>,
    raw_input_events: GenericReceiver<RawInputEvent>,
    input_events: GenericSender<InputEvent>,
    output_commands: GenericReceiver<OutputSwitch>,
    raw_output_commands: GenericSender<RawOutputSwitch>,
    tracer: trace_helper::TraceHelper,
    timer: Arc<Timer>,
    input_list: ModCapAggregator,
    output_list: Shareable<Vec<OutputEntry>>,
    dataevent: Arc<DataEvent<u32>>

}

impl Drop for IoManager
{
    fn drop(&mut self) {
        self.timer.stop();
    }
}

impl IoManager
{
    pub fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        IoManager{
            system_events_rx    : chm.get_receiver(),
            system_events_tx    : chm.get_sender(),
            modcaps_rx          : chm.get_receiver(),
            raw_input_events    : chm.get_receiver(),
            input_events        : chm.get_sender(),
            output_commands     : chm.get_receiver(),
            raw_output_commands : chm.get_sender(),
            tracer              : trace,
            timer               : Timer::new(),
            input_list          : ModCapAggregator::new(),
            output_list         : Shareable::new(Vec::new()),
            dataevent           : Arc::new(DataEvent::new("IOWait".to_string()))
        }
    }

    pub fn init(&self)
    {
        self.modcaps_rx.set_data_trigger(self.dataevent.clone(), 0);
        self.raw_input_events.set_data_trigger(self.dataevent.clone(), 1);
        self.output_commands.set_data_trigger(self.dataevent.clone(), 2);

        bootstage_helper::plain_boot(MODULE_ID, &self.system_events_tx, &self.system_events_rx, &self.tracer);
    }

    fn do_all_modcap_messages(&mut self)
    {
        // called upon HLI, all I/O modules must have advertised by now.
        while let Some(cap) = self.modcaps_rx.receive_with_timeout(0)
        {
            self.process_modcaps_message(cap);
        }

        self.modcaps_done();
    }

    pub fn process_modcaps_message(&mut self, message: crate::modcaps::ModuleCapabilityAdvertisement)
    {
        self.input_list.add_message(message);
    }

    pub fn modcaps_done(&mut self)
    {
        self.input_list.build();
        for _ in 0..self.input_list.get_num_entries(ModuleCapabilityType::Outputs)
        {
            self.output_list.lock().push(OutputEntry{timer_guard: None});
        }

    }

    pub fn run(&mut self) -> bool
    {
        self.tracer.trace_str("Waiting for commands");
        self.modcaps_rx.set_data_trigger(self.dataevent.clone(), 0);
        self.raw_input_events.set_data_trigger(self.dataevent.clone(), 1);
        self.output_commands.set_data_trigger(self.dataevent.clone(), 2);
        let chanid = self.dataevent.wait();

        match chanid
        {
            0 => {
                // Note: This should actually be done during HLI, however, if the
                // other modules advertise only during LLI this should work just as
                // well.
                self.do_all_modcap_messages();
            },
            1 => self.dispatch_raw_input_event(),
            2 => self.dispatch_output_command(),
            _ => return true
        }

        return true
    }

    fn dispatch_output_command(&self)
    {
        
        let command = self.output_commands.receive();
        self.tracer.trace(format!("Switching output {}", command.output_id));
        if let Ok(output) = self.input_list.logical_id_to_sud(command.output_id, ModuleCapabilityType::Outputs)        
        {
            // step 2: generate actual command:
            let raw_cmd = RawOutputSwitch{output_id: output, target_state: command.target_state.clone()};
            self.raw_output_commands.send(raw_cmd);

            // Drop the guard, preventing the timer
            // from triggering the reset.
            let mut output_access = self.output_list.lock();
            let output_entry = &mut output_access[command.output_id as usize];

            if output_entry.timer_guard.is_some()
            {
                output_entry.timer_guard = None;
            }

            if command.switch_time > 0
            {
                self.tracer.trace(format!("Schedule switchback in {} ms", command.switch_time));
                let sender = self.output_commands.create_sender();
                let switch_time = command.switch_time;
                let output_id = command.output_id;
                let g = self.timer.schedule(Box::new(move || {
                    let mut cmd = command.clone();
                    match cmd.target_state
                    {
                        OutputState::High => cmd.target_state = OutputState::Low,
                        OutputState::Low => cmd.target_state = OutputState::High
                    }
                    // permanent switchback;
                    cmd.switch_time = 0;
                    sender.send(cmd);
                }), switch_time);                
                output_access[output_id as usize].timer_guard = Some(g)
            }
        }
        else
        {
            self.tracer.trace_str("Invalid output.");
        }


    }

    fn dispatch_raw_input_event(&self)
    {
        let event = self.raw_input_events.receive();
        if let Ok(input_id) = self.input_list.sud_to_logical_id(event.input_id, ModuleCapabilityType::Inputs)
        {
            self.input_events.send(InputEvent {
                input_id: input_id as u32,
                state: event.state
            })
        }
    }

    // pub fn handle_put_input_setting(setting: InputSetting)
    // {}

    // pub fn handle_put_output_setting(setting: OutputSetting)
    // {}

    // pub fn handle_get_inputs() -> Vec<InputSetting>
    // {}

    // pub fn handle_get_outputs() -> Vec<OutputSetting>
    // {}

}

#[cfg(test)]
mod tests {

    /*
        Implement the following tests:
        * switch_output sends message with correct SUD
        * switch_output with bad ID doesn't crash
    */
    use barracuda_core::core::*;
    use crate::io::*;
    use crate::modcaps::{ModuleCapabilityAdvertisement, ModuleCapability};
    use std::time::Duration;


    fn make_mod() -> (IoManager, GenericSender<crate::io::RawInputEvent>,
                      GenericReceiver<crate::io::InputEvent>,
                      GenericSender<OutputSwitch>, GenericReceiver<crate::io::RawOutputSwitch>)
    {
        let mut chm = ChannelManager::new();
        let trace = trace_helper::TraceHelper::new("".to_string(), &mut chm);
        let sender = chm.get_sender::<crate::io::RawInputEvent>();
        let receiver = chm.get_receiver::<crate::io::InputEvent>();
        let output_sender = chm.get_sender::<crate::io::OutputSwitch>();
        let output_command_recv = chm.get_receiver::<crate::io::RawOutputSwitch>();
        let mut module = IoManager::new(trace, &mut chm);
        let modcap = ModuleCapabilityAdvertisement {module_id : make_sud(10, 0, 0), caps : vec![ModuleCapability::Inputs(4), ModuleCapability::Outputs(4)] };
        let modcap2 = ModuleCapabilityAdvertisement {module_id : make_sud(12, 0, 0), caps : vec![ModuleCapability::Inputs(2), ModuleCapability::Outputs(2)] };
        module.process_modcaps_message(modcap);
        module.process_modcaps_message(modcap2);
        module.modcaps_done();
        return (module, sender, receiver, output_sender, output_command_recv)
    }

    #[test]
    pub fn raw_input_event_id_is_converted_to_input_event()
    {
        let mut md = make_mod();
        let s = md.1;
        let evt = RawInputEvent {input_id: make_sud(10, 0, 1), state: InputState::High};
        s.send(evt);
        md.0.run();
        let recv = md.2.receive_with_timeout(1).unwrap();

        assert_eq!(recv.input_id, 1 )
    }

    #[test]
    pub fn raw_input_event_id_is_converted_to_input_event_from_second_module()
    {
        let mut md = make_mod();
        let s = md.1;
        let evt = RawInputEvent {input_id: make_sud(12, 0, 1), state: InputState::High};
        s.send(evt);
        md.0.run();
        let recv = md.2.receive_with_timeout(1).unwrap();

        assert_eq!(recv.input_id, 5 )
    }

    #[test]
    pub fn raw_input_event_with_unknown_source_is_ignored()
    {
        let mut md = make_mod();
        let s = md.1;
        let evt = RawInputEvent {input_id: make_sud(14, 0, 1), state: InputState::High};
        s.send(evt);
        md.0.run();
        let recv = md.2.receive_with_timeout(1);

        assert!(recv.is_none())
    }

    #[test]
    pub fn output_command_is_converted_to_raw_output_command()
    {
        let mut md = make_mod();
        let s = md.3;
        let evt = OutputSwitch {output_id: 1, target_state: OutputState::High, switch_time: 100};
        s.send(evt);
        md.0.run();
        let recv = md.4.receive_with_timeout(1).unwrap();

        assert_eq!(recv.output_id, make_sud(10, 0, 1))
    }

    #[test]
    pub fn output_command_sends_switchback()
    {
        for _ in 0..10
        {
            let mut md = make_mod();
            let s = md.3;
            let evt = OutputSwitch {output_id: 1, target_state: OutputState::High, switch_time: 100};
            s.send(evt);
            md.0.run();
            let recv = md.4.receive_with_timeout(1).unwrap();
            assert_eq!(recv.output_id, make_sud(10, 0, 1));
            // The internal timer will trigger the switchback after 100 ms, we'll wait
            // some more time to avoid a volatile test.
            thread::sleep(Duration::from_millis(100));
            md.0.run();
            let recv = md.4.receive_with_timeout(1).unwrap();
            assert_eq!(recv.output_id, make_sud(10, 0, 1));
            assert!(OutputState::Low == recv.target_state)
        }
    }


    #[test]
    pub fn output_command_with_unkown_target_is_ignored()
    {
        let mut md = make_mod();
        let s = md.3;
        let evt = OutputSwitch {output_id: 74, target_state: OutputState::High, switch_time: 100};
        s.send(evt);
        md.0.run();
        let recv = md.4.receive_with_timeout(1);

        assert!(recv.is_none());
    }


}