extern crate barracuda_core;

//extern crate barracuda_hal;

use barracuda_core::core::broadcast_channel::*;
use barracuda_core::core::channel_manager::*;
use barracuda_core::core::{shareable::Shareable, bootstage_helper::*, SystemMessage};
use barracuda_core::{Handler, cfg::{ConfigMessage, cfgholder::*, self}};
use barracuda_core::trace::*;
use barracuda_core::dcm::DoorOpenRequest;
use std::{sync::Arc, thread};

mod components;

const MODULE_ID: u32 = 0x03000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("DCM/ADCM".to_string(), chm);
    // let mut adcm = ADCM::new(tracer, chm);
    // thread::spawn(move || {  
    //     adcm.init();   
    //     loop 
    //     {
    //         if !adcm.do_request()
    //         {
    //             break;
    //         }
    //     }   
        
    // });
}

/*
    Data consumed by this module:
    * Door Open Requests
        -> By identification token
        -> By direct command
    * Input Events
    * Profile Changes

*/