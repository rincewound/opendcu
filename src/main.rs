#[macro_use]
//extern crate diesel;

mod core;
mod Trace;

//use crate::acm::WhitelistAccessRequest;


use self::core::BroadcastChannel::GenericReceiver;
use self::core::ChannelManager::ChannelManager;

// Modules
//mod acm;

fn main() {
    let mut chm = ChannelManager::new();
    chm.register_channel::<core::SystemMessage>();
    //chm.register_channel::<WhitelistAccessRequest>();

    Trace::launch(&mut chm);

    let tracer = Trace::TraceHelper::TraceHelper::new(String::from("Main"), &chm);
    tracer.TraceStr("..welcome to barracuda. Starting up.");

    let sysmsg = chm.get_receiver::<core::SystemMessage>().unwrap();

    launch_modules(&sysmsg, &chm);

    // After all is done, we wait for the shutdown signal
    
    loop {
        let event = sysmsg.receive();
        match event
        {
            core::SystemMessage::Shutdown => break,
            _ => continue
        }
    }
}

fn launch_modules(recv: &GenericReceiver<core::SystemMessage>, chm: &ChannelManager)
{
    let sender = chm.get_sender::<core::SystemMessage>().unwrap();
    // Step 1: Launch all modules
    //crate::acm::generic_whitelist::launch(chm);

    // Once all threads are go, send a message to the threads to actually start:
    sender.send(core::SystemMessage::RunStage(core::BootStage::LowLevelInit));
    wait_for_stage_completion(recv, core::BootStage::LowLevelInit, 1);

    // After lowlevel init is done, do the highlevel init
    sender.send(core::SystemMessage::RunStage(core::BootStage::HighLevelInit));
    wait_for_stage_completion(recv, core::BootStage::HighLevelInit, 1);

    // Now all modules should have the required data present for running without problems
    // and can enter the application stage.
    sender.send(core::SystemMessage::RunStage(core::BootStage::Application));
    // No need to wait here, this is where the rest of the application happens.
    //wait_for_stage_completion(recv, core::BootStage::Application, 0);
}

fn wait_for_stage_completion(recv: &GenericReceiver<core::SystemMessage>, stage: core::BootStage, num_participants: u32 ) -> bool
{
    let mut messages_left = num_participants;
    while messages_left > 0
    {
        let data = recv.receive_with_timeout(1000);
        if let Some(received) = data {
            match received
            {
                core::SystemMessage::StageComplete(the_stage) => if the_stage == stage {messages_left -= 1},
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
