/*
    The supervisor launches all modules and orchestrates the
    whole launch proces. It will also periodically ping any
    module, to make sure that it is still running correctly.

    desired api: launch!(acm::generic_whitelist::launch, trace::launch, othermodule::launch)
*/

use crate::core::{SystemMessage, BootStage};
use crate::core::BroadcastChannel::*;
use std::sync::Arc;

pub struct Supervisor{
    chm: crate::core::ChannelManager::ChannelManager,
    num_threads: u32
}

impl Supervisor
{
    pub fn new() -> Self{
        Supervisor
        {
            chm: crate::core::ChannelManager::ChannelManager::new(),
            num_threads: 0
        }
    }

    pub fn start_thread<T>(&mut self, launcher: T)
        where T: FnOnce(&crate::core::ChannelManager::ChannelManager)
    {
        self.num_threads += 1;
        launcher(&self.chm);
    }

    pub fn run(&self)
    {
        self.do_startup();
        let sysmsg = self.chm.get_receiver::<crate::core::SystemMessage>().unwrap();
        loop {
            let event = sysmsg.receive();
            match event
            {
                SystemMessage::Shutdown => break,
                _ => continue
            }
        }
    }

    fn do_startup(&self)
    {
        let recv = self.chm.get_receiver::<crate::core::SystemMessage>().unwrap();
        let sender = self.chm.get_sender::<SystemMessage>().unwrap();  
        // Once all threads are go, send a message to the threads to actually start:
        sender.send(SystemMessage::RunStage(BootStage::LowLevelInit));
        self.wait_for_stage_completion(&recv, BootStage::LowLevelInit, self.num_threads);
    
        // After lowlevel init is done, do the highlevel init
        sender.send(SystemMessage::RunStage(BootStage::HighLevelInit));
        self.wait_for_stage_completion(&recv, BootStage::HighLevelInit, self.num_threads);
    
        // Now all modules should have the required data present for running without problems
        // and can enter the application stage.
        sender.send(SystemMessage::RunStage(BootStage::Application));
        // No need to wait here, this is where the rest of the application happens.
        //wait_for_stage_completion(recv, core::BootStage::Application, 0);
    }

    fn wait_for_stage_completion(&self, recv: &Arc<GenericReceiver<SystemMessage>>, stage: BootStage, num_participants: u32 ) -> bool
    {
        let mut messages_left = num_participants;
        while messages_left > 0
        {
            let data = recv.receive_with_timeout(1000);
            if let Some(received) = data {
                match received
                {
                    SystemMessage::StageComplete(the_stage) => if the_stage == stage {messages_left -= 1},
                    _ => continue
                }

            }
            else
            {
                break;
            }
        }
        return messages_left <= 0 
    }
}