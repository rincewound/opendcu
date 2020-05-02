/*
    The supervisor launches all modules and orchestrates the
    whole launch proces. It will also periodically ping any
    module, to make sure that it is still running correctly.

    desired api: launch!(acm::generic_whitelist::launch, trace::launch, othermodule::launch)
*/

use crate::core::{SystemMessage, BootStage};
use crate::core::broadcast_channel::*;
use crate::trace::*;
use std::sync::Arc;

pub struct Supervisor{
    sysrec: Arc<crate::core::broadcast_channel::GenericReceiver<SystemMessage>>,
    tracer: trace_helper::TraceHelper,
    chm: crate::core::channel_manager::ChannelManager,
    num_threads: u32
}

impl Supervisor
{
    pub fn new() -> Self{
        let mut chanmgr = crate::core::channel_manager::ChannelManager::new();
        let syschan = chanmgr.get_receiver::<SystemMessage>();

        Supervisor
        { 
            sysrec: syschan,
            tracer: trace_helper::TraceHelper::new("SYS/Sypervisor".to_string(), &mut chanmgr),
            chm: chanmgr,            
            num_threads: 0
        }
    }

    pub fn start_thread<T>(&mut self, launcher: T)
        where T: FnOnce(&mut crate::core::channel_manager::ChannelManager)
    {
        self.num_threads += 1;
        launcher(&mut self.chm);
    }

    pub fn run(&mut self)
    {        
        self.tracer.trace_str("Starting system.");
        self.do_startup();        
        loop {
            let event = self.sysrec.receive();
            match event
            {
                SystemMessage::Shutdown => break,
                _ => continue
            }
        }
    }

    fn do_startup(&mut self)
    {                
        let sender = self.chm.get_sender::<SystemMessage>();  

        // All modules will send a sync message upon starup to signal that they are ready.
        self.tracer.trace_str("Wait for Sync");        
        self.wait_for_stage_completion(&self.sysrec, BootStage::Sync, self.num_threads);

        // Once all threads are go, send a message to the threads to actually start:
        self.tracer.trace_str("Bootstage: LowLevelInit");
        sender.send(SystemMessage::RunStage(BootStage::LowLevelInit));
        self.wait_for_stage_completion(&self.sysrec, BootStage::LowLevelInit, self.num_threads);
    
        // After lowlevel init is done, do the highlevel init
        self.tracer.trace_str("Bootstage: HighLevelInit");
        sender.send(SystemMessage::RunStage(BootStage::HighLevelInit));
        self.wait_for_stage_completion(&self.sysrec, BootStage::HighLevelInit, self.num_threads);
    
        // Now all modules should have the required data present for running without problems
        // and can enter the application stage.
        self.tracer.trace_str("Boot complete. Barracuda is ready.");
        sender.send(SystemMessage::RunStage(BootStage::Application));
        // No need to wait here, this is where the rest of the application happens.
        //wait_for_stage_completion(recv, core::BootStage::Application, 0);
    }

    fn wait_for_stage_completion(&self, recv: &Arc<GenericReceiver<SystemMessage>>, stage: BootStage, num_participants: u32 ) -> bool
    {
        let mut checked_in = Vec::<u32>::new();
        let mut messages_left = num_participants;
        while messages_left > 0
        {
            let data = recv.receive_with_timeout(2500);
            if let Some(received) = data {
                match received
                {
                    SystemMessage::StageComplete(the_stage, mod_id) => 
                    {
                        if std::mem::discriminant(&the_stage) == std::mem::discriminant(&stage) {
                            let mod_type = (mod_id & 0xFF000000) >> 24;
                            let mod_instance = mod_id & 0x00FF0000 >> 16;
                            self.tracer.trace(format!("Module {}, instance {} checked in for stage {} ", mod_type, mod_instance, the_stage as u32));
                            checked_in.push(mod_id);
                            messages_left -= 1
                        }                     
                    }
                    _ => continue
                }

            }
            else
            {
                break;
            }
        }

        if messages_left > 0
        {
            let msg = format!("{} modules failed to run stage {}", messages_left, stage as u32);
            println!("{}", msg);
            for i in checked_in.iter()
            {
                let mod_type = (i &0xFF000000) >> 24;
                let mod_instance = i & 0x00FF0000 >> 16;
                println!("module {}, instance {} checked in", mod_type, mod_instance);
            }
            panic!("Failed to boot. Not all modules checked in.")
        }

        return messages_left <= 0 
    }
}