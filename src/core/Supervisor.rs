/*
    The supervisor launches all modules and orchestrates the
    whole launch proces. It will also periodically ping any
    module, to make sure that it is still running correctly.

    desired api: launch!(acm::generic_whitelist::launch, trace::launch, othermodule::launch)
*/

use crate::core::{SystemMessage, BootStage};
use crate::core::BroadcastChannel::*;
use crate::Trace::TraceHelper::TraceHelper;
use std::sync::Arc;

pub struct Supervisor{
    sysrec: Arc<crate::core::BroadcastChannel::GenericReceiver<SystemMessage>>,
    chm: crate::core::ChannelManager::ChannelManager,
    num_threads: u32
}

impl Supervisor
{
    pub fn new() -> Self{
        let mut chanmgr = crate::core::ChannelManager::ChannelManager::new();
        let syschan = chanmgr.get_receiver::<SystemMessage>();
        Supervisor
        { 
            sysrec: syschan,
            chm: chanmgr,            
            num_threads: 0
        }
    }

    pub fn start_thread<T>(&mut self, launcher: T)
        where T: FnOnce(&mut crate::core::ChannelManager::ChannelManager)
    {
        self.num_threads += 1;
        launcher(&mut self.chm);
    }

    pub fn run(&mut self)
    {
        let tracer = crate::Trace::TraceHelper::TraceHelper::new("SYS/Sypervisor".to_string(), &mut self.chm);
        tracer.TraceStr("Starting system.");
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
        let tracer = crate::Trace::TraceHelper::TraceHelper::new("SYS/Sypervisor".to_string(), &mut self.chm);
        let sender = self.chm.get_sender::<SystemMessage>();  

        // All modules will send a sync message upon starup to signal that they are ready.
        tracer.TraceStr("Wait for Sync");        
        self.wait_for_stage_completion(&self.sysrec, BootStage::Sync, self.num_threads);

        // Once all threads are go, send a message to the threads to actually start:
        tracer.TraceStr("Bootstage: LowLevelInit");
        sender.send(SystemMessage::RunStage(BootStage::LowLevelInit));
        self.wait_for_stage_completion(&self.sysrec, BootStage::LowLevelInit, self.num_threads);
    
        // After lowlevel init is done, do the highlevel init
        tracer.TraceStr("Bootstage: HighLevelInit");
        sender.send(SystemMessage::RunStage(BootStage::HighLevelInit));
        self.wait_for_stage_completion(&self.sysrec, BootStage::HighLevelInit, self.num_threads);
    
        // Now all modules should have the required data present for running without problems
        // and can enter the application stage.
        tracer.TraceStr("Boot complete. Barracuda is ready.");
        sender.send(SystemMessage::RunStage(BootStage::Application));
        // No need to wait here, this is where the rest of the application happens.
        //wait_for_stage_completion(recv, core::BootStage::Application, 0);
    }

    fn wait_for_stage_completion(&self, recv: &Arc<GenericReceiver<SystemMessage>>, stage: BootStage, num_participants: u32 ) -> bool
    {
        let mut messages_left = num_participants;
        while messages_left > 0
        {
            let data = recv.receive_with_timeout(2500);
            if let Some(received) = data {
                match received
                {
                    SystemMessage::StageComplete(the_stage) => if the_stage == stage {messages_left -= 1},
                    _ => continue
                }

            }
            else
            {
                println!("..still waiting.");
                break;
            }
        }

        if messages_left > 0
        {
            panic!("{} modules failed to run stage {} ", messages_left, stage as u32)
        }

        return messages_left <= 0 
    }
}