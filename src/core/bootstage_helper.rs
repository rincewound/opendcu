use crate::core::{SystemMessage, BootStage};
use crate::core::broadcast_channel::*;
use crate::trace::trace_helper::TraceHelper;
use std::{mem, sync::Arc};


pub fn plain_boot(module_id: u32, sys_chan: GenericSender<SystemMessage>, sys_chan_rx: Arc<GenericReceiver<SystemMessage>>, tracer: &TraceHelper)
{
    tracer.trace_str("Starting");
    send_stage_complete(module_id, BootStage::Sync, &sys_chan);

    wait_for_stage(BootStage::LowLevelInit, &sys_chan_rx, tracer);
    tracer.trace_str("Runstage: LLI");
    send_stage_complete(module_id, BootStage::LowLevelInit, &sys_chan);

    wait_for_stage(BootStage::HighLevelInit, &sys_chan_rx, tracer);
    tracer.trace_str("Runstage: HLI");
    send_stage_complete(module_id, BootStage::HighLevelInit, &sys_chan);

    wait_for_stage(BootStage::Application, &sys_chan_rx, tracer);
    tracer.trace_str("Runstage: APP");
}

fn try_trigger_stage_cb<Fun>(boot_stage: BootStage, stage_cb: &mut [Option<Fun>; 2])
    where Fun: FnOnce() -> ()
{
    let func = mem::replace(&mut stage_cb[boot_stage as usize], std::option::Option::<Fun>::None);
    if let Some(f) = func
    {
        f();
    }
}

/*
*  This function will return a
*  "stage complete" for all stages,
*  making it easier to boot modules
*  that have no external dependencies.
*/
pub fn boot<Fun>(module_id: u32, mut stage_cb: [Option<Fun>; 2], sys_chan: GenericSender<SystemMessage>, sys_chan_rx: Arc<GenericReceiver<SystemMessage>>, tracer: &TraceHelper)
    where Fun: FnOnce() -> ()
{
    tracer.trace_str("Starting");
    send_stage_complete(module_id, BootStage::Sync, &sys_chan);        

    wait_for_stage(BootStage::LowLevelInit, &sys_chan_rx, tracer);
    tracer.trace_str("Runstage: LLI");
    try_trigger_stage_cb(BootStage::LowLevelInit, &mut stage_cb);    
    send_stage_complete(module_id, BootStage::LowLevelInit, &sys_chan);

    wait_for_stage(BootStage::HighLevelInit, &sys_chan_rx, tracer);
    tracer.trace_str("Runstage: HLI");
    try_trigger_stage_cb(BootStage::HighLevelInit, &mut stage_cb);
    send_stage_complete(module_id, BootStage::HighLevelInit, &sys_chan);

    wait_for_stage(BootStage::Application, &sys_chan_rx, tracer);
    tracer.trace_str("Runstage: APP");
}

fn send_stage_complete(module_id: u32, stage: BootStage, sys_chan: &GenericSender<SystemMessage>)
{
    sys_chan.send(SystemMessage::StageComplete(stage, module_id));
}

fn wait_for_stage(stage: BootStage, sys_chan_rx: &Arc<GenericReceiver<SystemMessage>>, tracer: &TraceHelper)
    {
        tracer.trace(format!("Wait for stage signal {}", stage as u32));
        loop
        {
            let msg = sys_chan_rx.receive();
            match msg
            {
                SystemMessage::RunStage(s) => if s == stage {
                    break;
                },
                _ => continue /*ABORTS!*/
            }
        }  
    }