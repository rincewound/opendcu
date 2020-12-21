use std::{sync::Arc, thread};

use crate::{cfg::{self, ConfigMessage, cfgholder::FunctionType}, core::{bootstage_helper::{boot, boot_noop}, broadcast_channel::{GenericReceiver, GenericSender}, channel_manager::ChannelManager}, trace::trace_helper};


const MODULE_ID: u32 = 0x0C000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("ProfileControl".to_string(), chm);
    let mut wl = SignalControl::new(tracer, chm);
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


struct SignalControl
{
    tracer: trace_helper::TraceHelper,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    // profile_state_tx: GenericSender<ProfileChangeEvent>,
    cfg_rx          : Arc<GenericReceiver<ConfigMessage>>,
    // checker         : Shareable<profile_checker::ProfileChecker>
}

impl SignalControl
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        SignalControl
        {
            tracer: trace,
            system_events_rx: chm.get_receiver(),
            system_events_tx: chm.get_sender(),
            //profile_state_tx: chm.get_sender(),
            cfg_rx:           chm.get_receiver(),
            //checker:          Shareable::new(profile_checker::ProfileChecker::new())
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

            // let res = the_receiver.receive();
            // let cfg::ConfigMessage::RegisterHandlers(cfg_holder) = res;
            // let mut holder = cfg_holder.lock();
            // let profile_writer = self.checker.clone();
            // let profile_deleter = self.checker.clone();

            // holder.register_handler(FunctionType::Put, "sigcontrol/entry".to_string(), Handler!(|r: profile_checker::BinaryProfile|
            //     {
            //         //profile_writer.lock().add_profile(r);
            //     }));

            // holder.register_handler(FunctionType::Delete, "profiles/entry".to_string(), Handler!(|r: profile_checker::BinaryProfile|
            //     {
            //         //profile_deleter.lock().delete_profile(r.id);
            //     }));            
        });

        boot(MODULE_ID, Some(boot_noop), hli_cb, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);

    }

    pub fn run(&mut self) -> bool
    {
        //let mut last_date_time = Local::now();
        loop 
        {
            // if let Some(e) = self.system_events_rx.receive_with_timeout(5000)
            // {
            //     if e == SystemMessage::Shutdown
            //     {
            //         return false
            //     }
            // }

            // let current_time = Local::now();
            // let events = self.checker.lock().tick(current_time, last_date_time);

            // last_date_time = current_time;
            // for e in events.into_iter()
            // {
            //     self.profile_state_tx.send(e);
            // }
        }
    }
}