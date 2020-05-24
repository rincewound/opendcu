extern crate ws;
use ws::listen;

use crate::{webserver,
            trace::trace_helper,
            core::{broadcast_channel::{GenericSender, GenericReceiver}, channel_manager::ChannelManager}
           };

use std::{thread, sync::Arc};


const MODULE_ID: u32 = 0x05500000;
const ADDR: &str = "127.0.0.1:3013";

pub fn launch(chm: &mut ChannelManager)
{
    let socket = Websocket::new(chm);
    let handle = thread::spawn(move || {
        socket.work();
    });

}

struct Websocket 
{
    tracer: trace_helper::TraceHelper,
    door_rx: Arc<GenericReceiver<crate::dcm::DoorOpenRequest>>,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
}

impl Websocket 
{
    pub fn new(chm: &mut ChannelManager) -> Self
    {
        let tracer_helper = trace_helper::TraceHelper::new("WebSocket/server".to_string(), chm);
        Websocket
        {
            tracer: tracer_helper,
            door_rx: chm.get_receiver::<crate::dcm::DoorOpenRequest>(),
            system_events_rx: chm.get_receiver::<crate::core::SystemMessage>(),
            system_events_tx: chm.get_sender::<crate::core::SystemMessage>(),
        }
    } 

    fn init(&self)
    {
        crate::core::bootstage_helper::plain_boot(MODULE_ID,
            self.system_events_tx.clone(),
            self.system_events_rx.clone(),
            &self.tracer);
    }

    fn run(&self) -> bool
    {
        if let Err(error) =  listen(ADDR, |out| {
            move |msg| {
                println!("Server got a Message: '{}'", msg);
                out.send("send message back")
            }
        }) {
            println!("failed to create websocket: '{:?}'", error);
        }
        
        loop
        {
            let received = self.door_rx.receive();
            println!("received: {:?}", received.access_point_id);
        }

        true
    }

    pub fn work(&self)
    {
        self.init();
        self.run();
    }

}