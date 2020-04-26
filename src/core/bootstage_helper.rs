use crate::core::{SystemMessage, BootStage};
use crate::core::broadcast_channel::*;
use crate::trace::trace_helper::TraceHelper;
use std::sync::Arc;

/*
*  This function will return a
*  "stage complete" for all stages,
*  making it easier to boot modules
*  that have no external dependencies.
*/
pub fn boot(module_id: u32, sys_chan: GenericSender<crate::core::SystemMessage>, sys_chan_rx: Arc<GenericReceiver<crate::core::SystemMessage>>, tracer: &TraceHelper)
{
    tracer.TraceStr("Starting");
    send_stage_complete(module_id, BootStage::Sync, &sys_chan);

    wait_for_stage(BootStage::LowLevelInit, &sys_chan_rx, tracer);
    tracer.TraceStr("Runstage: LLI");
    send_stage_complete(module_id, BootStage::LowLevelInit, &sys_chan);

    wait_for_stage(BootStage::HighLevelInit, &sys_chan_rx, tracer);
    tracer.TraceStr("Runstage: HLI");
    send_stage_complete(module_id, BootStage::HighLevelInit, &sys_chan);

    wait_for_stage(BootStage::Application, &sys_chan_rx, tracer);
    tracer.TraceStr("Runstage: APP");
}

fn send_stage_complete(module_id: u32, stage: BootStage, sys_chan: &GenericSender<crate::core::SystemMessage>)
{
    sys_chan.send(crate::core::SystemMessage::StageComplete(stage, module_id));
}

fn wait_for_stage(stage: BootStage, sys_chan_rx: &Arc<GenericReceiver<crate::core::SystemMessage>>, tracer: &TraceHelper)
    {
        tracer.Trace(format!("Wait for stage signal {}", stage as u32));
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