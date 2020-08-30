use crate::core::broadcast_channel::*;
use crate::core::{bootstage_helper::{boot, boot_noop}, channel_manager::*, shareable::Shareable};
use crate::{cfg::{cfgholder::FunctionType, ConfigMessage}, trace::*};
use crate::{Handler, cfg::{cfgholder::*, self}};
use std::{sync::Arc, thread};
mod profile_checker;

use chrono::{DateTime, Local};

const MODULE_ID: u32 = 0x0C000000;

#[derive(Clone)]
pub enum ProfileState
{
    Active,
    Inactive
}

#[derive(Clone)]
pub struct ProfileChangeEvent
{
    profile_id: u32,
    profile_state: ProfileState
}

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("ProfileControl".to_string(), chm);
    let mut wl = ProfileControl::new(tracer, chm);
    thread::spawn(move || {  
        wl.init();   
        loop 
        {
            if !wl.run()
            {
                break;
            }
        }   
        
    });
}

#[derive()]
struct ProfileControl
{
    tracer: trace_helper::TraceHelper,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    profile_state_tx: GenericSender<ProfileChangeEvent>,
    cfg_rx          : Arc<GenericReceiver<ConfigMessage>>,
    checker         : Shareable<profile_checker::ProfileChecker>
}

impl ProfileControl
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        ProfileControl
        {
            tracer: trace,
            system_events_rx: chm.get_receiver(),
            system_events_tx: chm.get_sender(),
            profile_state_tx: chm.get_sender(),
            cfg_rx:           chm.get_receiver(),
            checker:          Shareable::new(profile_checker::ProfileChecker::new())
        }
    }

    pub fn init(&mut self)
    {
        //crate::core::bootstage_helper::plain_boot(MODULE_ID, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer);        
        let the_receiver = self.cfg_rx.clone();  
        let hli_cb= Some(|| {
            /*
                This is executed during HLI
            */

            let res = the_receiver.receive();
            let cfg::ConfigMessage::RegisterHandlers(cfg_holder) = res;
            let mut holder = cfg_holder.lock();
            let profileWriter = self.checker.clone();
            let profileDeleter = self.checker.clone();

            holder.register_handler(FunctionType::Put, "profiles/entry".to_string(), Handler!(|r: profile_checker::BinaryProfile|
                {
                    profileWriter.lock().add_profile(r);
                }));

            holder.register_handler(FunctionType::Delete, "profiles/entry".to_string(), Handler!(|r: profile_checker::BinaryProfile|
                {
                    profileDeleter.lock().delete_profile(r.id);
                }));            
        });

        boot(MODULE_ID, Some(boot_noop), hli_cb, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);

        //self.modcaps.aggregate(&self.modcap_rx);

    }

    pub fn run(&mut self) -> bool
    {
        let mut last_date_time = Local::now();
        loop 
        {
            if let Some(_) = self.system_events_rx.receive_with_timeout(5000)
            {
                // might have to break here, if a terminate command was received.
            }

            let current_time = Local::now();
            let events = self.checker.lock().tick(current_time, last_date_time);

            last_date_time = current_time;
            for e in events.into_iter()
            {
                self.profile_state_tx.send(e);
            }
        }
        false
    }

    // pub fn do_request(&mut self) -> bool
    // {

    //     let mut input = String::new();
    //     match io::stdin().read_line(&mut input) {
    //         Ok(_) => {
    //             let req = WhitelistAccessRequest
    //             {
    //                 access_point_id: MODULE_ID | 0x01,      // Access point 1 
    //                 identity_token_number: input.into_bytes()
    //             };
    //             self.access_request_tx.send(req);
    //         }
    //         Err(error) => println!("error: {}", error),
    //     }
    //     true
    // }
}