extern crate barracuda_core;

use std::{sync::Arc, thread, error::Error, time::Duration};
use rppal::gpio::Gpio;
use std::io;
use barracuda_core::{io::{OutputState, RawOutputSwitch},
                    trace::trace_helper,
                    core::{broadcast_channel::{GenericReceiver, GenericSender},
                           channel_manager::ChannelManager,
                           event::DataEvent},
                           modcaps::{ModuleCapability, ModuleCapabilityAdvertisement},
                        };


use barracuda_core::core;
use barracuda_core::launch;
use barracuda_core::wait_for;
use barracuda_core::select_chan;
use barracuda_core::launch_impl;

const GPIO: u8 = 23;


// TODO: remove test code
fn gpio()
{ 
    let mut gpio = Gpio::new().unwrap().get(GPIO).unwrap().into_output();
    gpio.set_high();
}

pub fn launch(chm: &mut ChannelManager)   
{    
    let tracer = trace_helper::TraceHelper::new("IO/IoModule".to_string(), chm);
    let mut io = IoModule::new(tracer, chm);
    thread::spawn(move || {  
        io.init();   
        loop 
        {
            if !io.run()
            {
                break;
            }
        }   
    });
}


const MODULE_ID: u32 = 0x17000000;

struct IoModule
{
    output: u8,
    input: u8,
    tracer: trace_helper::TraceHelper,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    switch_out_req: Arc<GenericReceiver<RawOutputSwitch>>,
    modcaps_tx:  GenericSender<barracuda_core::modcaps::ModuleCapabilityAdvertisement>,
}

impl IoModule
{
    pub fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        IoModule {output: 0,
                  input: 0,
                  tracer: trace,
                  system_events_rx: chm.get_receiver(),
                  system_events_tx: chm.get_sender(),
                  switch_out_req: chm.get_receiver::<RawOutputSwitch>(),
                  modcaps_tx: chm.get_sender(),
                }
    }

    pub fn init(&self)
    {
        let modcaps_tx_clone =self.modcaps_tx.clone();
        let llicb= Some(move|| {
            // TODO: num of inputs and outputs should be configured via web-ui
            let m = ModuleCapabilityAdvertisement {
                caps: vec![ModuleCapability::Outputs(3), ModuleCapability::Inputs(3)],
                module_id: MODULE_ID
            };
            modcaps_tx_clone.send(m);            
        });
        crate::core::bootstage_helper::plain_boot(MODULE_ID, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer);
    }

    fn set_output(&self, gpio_id: u8, state: OutputState)
    {
        let mut gpio = Gpio::new().unwrap().get(gpio_id).unwrap().into_output();
        match state
        {
            OutputState::Low =>  { 
                self.tracer.trace_str("switch output Low");
                gpio.set_low();
            },
            OutputState::High => {
                self.tracer.trace_str("switch output High");
                gpio.set_high();
            },
            _ => {}
        }
    }

    pub fn run(&self) -> bool
    {
        let queue = select_chan!(self.switch_out_req);
        match queue
        {
            0 => {
                let res = self.switch_out_req.receive();
                // TODO: remove example code
                self.set_output(GPIO, OutputState::Low);
                println!("io_request: {}", res.output_id)
                },
            _ => {}
        }
 
        false
    }
}
