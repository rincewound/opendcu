use std::thread;
use std::{sync::Arc};
use serde::{Serialize};

use barracuda_core::{core::{bootstage_helper::boot_noop, broadcast_channel::GenericReceiver, channel_manager::ChannelManager, event::DataEvent, shareable::Shareable}, select_chan, trace::trace_helper::{self, TraceHelper}, wait_for};

use crate::{cfg::{self, ConfigMessage}, modulebase::ModuleBase};

#[derive( Clone, Debug, PartialEq, Serialize)]
pub enum LogEvent
{
    AccessGranted(u32, Vec<u8>, u32),                  // pwayid, token, ap id
    AccessDeniedTimezoneViolated(u32, Vec<u8>, u32),   // pwayid, token, ap id
    AccessDeniedTokenUnknown(u32, Vec<u8>, u32),       // pwayid, token, ap id
    AccessDeniedDoorBlocked(u32, Vec<u8>, u32),        // pwayid, token, ap id

    DoorEmergencyReleased(u32),      // pwayid
    DoorEnteredNormalOperation(u32), // pwayid
    DoorPermantlyReleased(u32),      // pwayid
    DoorReleasedOnce(u32),  // pwayid
    DoorBlocked(u32),       // pwayid
    DoorForcedOpen(u32),    // pwayid
    DoorOpenTooLong(u32),   // pwayid
    DoorClosedAgain(u32),   // pwayid
}

const MODULE_ID: u32 = 0x0E000000;

pub fn launch(chm: &mut ChannelManager)
{        
    let tracer = trace_helper::TraceHelper::new("FDB/Events".to_string(), chm);
    let cr = EventStore::new(tracer, chm);      
    thread::spawn(move|| {
        cr.init();  
        cr.run();         
    });
}

pub struct EventStore
{
    module   : ModuleBase,
    event_rx : GenericReceiver<LogEvent>,
    cfg_rx   : GenericReceiver<ConfigMessage>,
    event_buffer: Shareable<Vec<LogEvent>>
}

impl EventStore {
    pub fn new(tracer: TraceHelper, chm: &mut ChannelManager) -> Self { 
        Self 
        {
         module          : ModuleBase::new(MODULE_ID, tracer, chm),
         event_rx        : chm.get_receiver(),
         cfg_rx          : chm.get_receiver(),
         event_buffer    : Shareable::new(Vec::new())
        } 
    }

    pub fn init(&self)
    {  
        let the_receiver = self.cfg_rx.clone_receiver();  
    
        let hli_cb= Some(|| {

         let event_buffer_access = self.event_buffer.clone();   

         let res = the_receiver.receive();
         let cfg::ConfigMessage::RegisterHandlers(cfg_holder) = res;
         let mut holder = cfg_holder.lock();
         holder.register_read_handler("events".to_string(), ReadDataHandler!(||
            {                 
                let mut data_access = event_buffer_access.lock();
                let items_to_take = if data_access.len() < 20 {data_access.len() }else {20};
                let items = &data_access[0..items_to_take];
                let mut items_to_return: Vec<LogEvent> = Vec::new();
                items_to_return.extend_from_slice(items);

                // remove taken items from internal buffer:
                for _ in 0..items_to_take
                {
                    data_access.swap_remove(0);
                }
                items_to_return
                
            }));         
        });
        self.module.boot(Some(boot_noop), hli_cb);
    }

    pub fn run(&self)
    {
        let _ = select_chan!(self.event_rx);
        let mut data_access = self.event_buffer.lock();
        while self.event_rx.has_data()
        {
            data_access.push(self.event_rx.receive())
        }
    }
}
