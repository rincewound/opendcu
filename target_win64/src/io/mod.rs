use barracuda_base_modules::{io::RawOutputSwitch, modcaps::{ModuleCapability, ModuleCapabilityAdvertisement}};
use barracuda_core::{
    core::{
        bootstage_helper::{boot, boot_noop},
        broadcast_channel::{GenericReceiver, GenericSender},
        channel_manager::ChannelManager,
    }, select_chan, trace::trace_helper
};
use std::thread;
use barracuda_core::core::event::DataEvent;
use barracuda_core::wait_for;
use std::sync::Arc;

const MODULE_ID: u32 = 0x09000000;

pub fn launch(chm: &mut ChannelManager, instance: u32) {
    let tracer = trace_helper::TraceHelper::new(format!("Plattform/Win32Io({})", instance), chm);
    let ioman = W32Io::new(tracer, chm);
    thread::spawn(move || {
        ioman.init(instance);
        loop {
            if !ioman.run() {
                break;
            }
        }
    });
}

struct W32Io {
    system_events_rx: GenericReceiver<barracuda_core::core::SystemMessage>,
    system_events_tx: GenericSender<barracuda_core::core::SystemMessage>,
    modcaps_tx: GenericSender<ModuleCapabilityAdvertisement>,
    output_cmd_rx: GenericReceiver<RawOutputSwitch>,
    tracer: trace_helper::TraceHelper,
}

impl W32Io {
    pub fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self {
        W32Io {
            system_events_rx: chm.get_receiver(),
            system_events_tx: chm.get_sender(),
            modcaps_tx: chm.get_sender(),
            output_cmd_rx: chm.get_receiver(),
            tracer: trace,
        }
    }

    pub fn init(&self, instance: u32) {
        let modcaps_tx_clone = self.modcaps_tx.clone();
        let llicb = Some(move || {
            /*
                This is executed during LLI
            */
            let m = ModuleCapabilityAdvertisement {
                caps: vec![ModuleCapability::Outputs(3), ModuleCapability::Inputs(3)],
                // let mod_instance = mod_id & 0x00FF0000 >> 16;
                module_id: MODULE_ID ^ (instance << 16) ,                
            };
            modcaps_tx_clone.send(m);
        });

        boot(
            MODULE_ID ^ (instance << 16),
            llicb,
            Some(boot_noop),
            &self.system_events_tx,
            &self.system_events_rx,
            &self.tracer,
        );
    }
    
    fn run(&self) -> bool {
        let queue = select_chan!(self.output_cmd_rx, self.system_events_rx);

        match queue {
            0 => {
                let res = self.output_cmd_rx.receive();
                
                self.tracer.trace(format!("io_request, swtitching output {} to {}", !MODULE_ID & res.output_id, res.target_state));
            }
            1 => {
                let res = self.system_events_rx.receive();
                match res {
                    barracuda_core::core::SystemMessage::Shutdown => todo!(),
                    barracuda_core::core::SystemMessage::StageComplete(boot_stage, _) => {},
                    barracuda_core::core::SystemMessage::RunStage(boot_stage) => todo!(),
                    barracuda_core::core::SystemMessage::_Reboot(_) => todo!(),
                    barracuda_core::core::SystemMessage::_Heartbeat => todo!(),
                    barracuda_core::core::SystemMessage::_HeartbeatResponse(_) => todo!(),
                }
            }
            _ => {}
        }

        true
    }
}
