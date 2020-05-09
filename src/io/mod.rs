
use crate::{trace::trace_helper, core::{broadcast_channel::{GenericReceiver, GenericSender}, channel_manager::ChannelManager}};
use std::{sync::Arc, thread};
use crate::core::event::DataEvent;
use crate::core::*;
use crate::modcaps::*;

#[derive(Clone)]
pub enum InputState
{
    Unknown,
    Low,
    High,
    Short,
    Cutout
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
    input_id: u32,      // Logical!
    state: InputState
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
pub struct InputSetting
{
    input_id: u32,              //Logical!
    inverted_polarity: bool,
    debounce_on: u64,
    debounce_off: u64
}

#[derive(Clone)]
pub enum OutputState
{
    Low,
    High
}

pub struct OutputSetting
{
    output_id: u32, // Logical!
    inverted_polarity: bool
}

#[derive(Clone)]
pub struct RawOutputSwitch
{
    output_id: u32,     // SUD!
    target_state: OutputState   // physical!
}

#[derive(Clone)]
pub struct OutputSwitch
{
    output_id: u32,
    target_state: OutputState,   //logical!
    switch_time: u64            // in ms!
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

struct InputEntry
{
    //id: u32,
    sud: u32
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
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    modcaps_rx: Arc<GenericReceiver<crate::modcaps::ModuleCapabilityAdvertisement>>,
    raw_input_events: Arc<GenericReceiver<RawInputEvent>>,
    input_events: GenericSender<InputEvent>,
    output_commands: Arc<GenericReceiver<OutputSwitch>>,
    raw_output_commands: GenericSender<RawOutputSwitch>,
    tracer: trace_helper::TraceHelper,
    input_list: Vec<InputEntry>,
}

impl IoManager
{
    pub fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        IoManager{            
            system_events_rx    : chm.get_receiver::<crate::core::SystemMessage>(),
            system_events_tx    : chm.get_sender::<crate::core::SystemMessage>(),
            modcaps_rx          : chm.get_receiver::<crate::modcaps::ModuleCapabilityAdvertisement>(),
            raw_input_events    : chm.get_receiver::<RawInputEvent>(),
            input_events        : chm.get_sender::<InputEvent>(),
            output_commands     : chm.get_receiver::<OutputSwitch>(),
            raw_output_commands : chm.get_sender::<RawOutputSwitch>(),
            tracer              : trace,
            input_list          : Vec::new()
        }
    } 

    pub fn init(&self)
    {
        crate::core::bootstage_helper::plain_boot(MODULE_ID, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer)
    }

    fn do_all_modcap_messages(&mut self)
    {
        // called upon HLI, all I/O modules must have advertised by now.
        while let Some(cap) = self.modcaps_rx.receive_with_timeout(0)
        {
            self.process_modcaps_message(cap);
        }
        self.input_list.sort_by(|a,b| a.sud.cmp(&b.sud));
    }

    pub fn process_modcaps_message(&mut self, message: crate::modcaps::ModuleCapabilityAdvertisement)
    {
        for x in message.caps
        {
            match x
            {
                ModuleCapability::Inputs(ins) => {
                    for i in 0..ins
                    {
                        self.input_list.push(InputEntry {sud: message.module_id | i});
                    }
                }
                _ => continue
            }
        }

    }

    pub fn run(&self) -> bool
    {
        // get minimum timeout of all pending switch commands and
        // all pending debounce events.

        // wait for either new raw events or the timeout
        let chanid = select_chan!(self.raw_input_events, self.output_commands);

        match chanid
        {
            0 => self.dispatch_raw_input_event(),
            1 => self.dispatch_output_command(),
            _ => return true
        }
        return true
    }

    fn dispatch_output_command(&self)
    {
        let command = self.output_commands.receive();

        // step 1: obtain SUD for output
        // step 2: generate actual command:
        let raw_cmd = RawOutputSwitch{output_id: command.output_id, target_state: command.target_state};
        self.raw_output_commands.send(raw_cmd);

        // step 3: store the switchtime

    }
    
    fn dispatch_raw_input_event(&self)
    {
        let event = self.raw_input_events.receive();

        // convert sud to logical id:
        let modcapentry = self.input_list
                                                            .iter()
                                                            .enumerate()
                                                            .find(|x| x.1.sud.eq(&event.input_id));
        if modcapentry.is_none()
        {
            return;
        }

        let (index, _) = modcapentry.unwrap();
        // update physical state and reset debounce time

        self.input_events.send(InputEvent {
            input_id: index as u32,
            state: event.state
        })
    }

    pub fn handle_put_input_setting(setting: InputSetting)
    {}

    pub fn handle_put_output_setting(setting: OutputSetting)
    {}

    // pub fn handle_get_inputs() -> Vec<InputSetting>
    // {}

    // pub fn handle_get_outputs() -> Vec<OutputSetting>
    // {}
    
}

#[cfg(test)]
mod tests {

    /*
        Implement the following tests:
        * can process a single modcaps message
        * multiple modcaps messages cause correct alignment of IOs
        * switch_output sends message with correct SUD
        * switch_output with bad ID doesn't crash
        * module will debounce input changes correctly
        * debounce times are observed
    */    
    use crate::core::*;
    use crate::io::*;
    use crate::modcaps::{ModuleCapabilityAdvertisement, ModuleCapability};


    fn make_mod() -> (IoManager, GenericSender<crate::io::RawInputEvent>, Arc<GenericReceiver<crate::io::InputEvent>>, GenericSender<OutputSwitch>)
    {
        let mut chm = crate::core::channel_manager::ChannelManager::new();
        let trace = trace_helper::TraceHelper::new("".to_string(), &mut chm);
        let sender = chm.get_sender::<crate::io::RawInputEvent>();
        let receiver = chm.get_receiver::<crate::io::InputEvent>();
        let output_sender = chm.get_sender::<crate::io::OutputSwitch>();
        let mut module = IoManager::new(trace, &mut chm);
        let modcap = ModuleCapabilityAdvertisement {module_id : make_sud(10, 0, 0), caps : vec![ModuleCapability::Inputs(4), ModuleCapability::Outputs(4)] };
        let modcap2 = ModuleCapabilityAdvertisement {module_id : make_sud(12, 0, 0), caps : vec![ModuleCapability::Inputs(2), ModuleCapability::Outputs(2)] };
        module.process_modcaps_message(modcap);
        module.process_modcaps_message(modcap2);
        return (module, sender, receiver, output_sender)
    }

    #[test]
    pub fn raw_input_event_id_is_converted_to_input_event()
    {
        let md = make_mod();
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
        let md = make_mod();
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
        let md = make_mod();
        let s = md.1;
        let evt = RawInputEvent {input_id: make_sud(14, 0, 1), state: InputState::High};
        s.send(evt);
        md.0.run();
        let recv = md.2.receive_with_timeout(1);

        assert!(recv.is_none())
    }
    
    
}