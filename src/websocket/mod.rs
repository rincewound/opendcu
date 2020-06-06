extern crate ws;
use ws::{listen, CloseCode, connect, Sender, Handler, Message, Result, Handshake};

use crate::{webserver,
            trace::trace_helper,
            core::{broadcast_channel::{GenericSender, GenericReceiver}, channel_manager::ChannelManager, event::DataEvent}
           };

use std::{thread, sync::Arc};


const MODULE_ID: u32 = 0x05500000;   // TODO: not sure about using this MODULE_ID 
const ADDR: &str = "127.0.0.1:3013";

pub fn launch(chm: &mut ChannelManager)
{
    let socket = Websocket::new(chm);
    thread::spawn(move || {
        socket.work();
    });
}


fn send_io(sender: Sender,
           io_rx: Arc<GenericReceiver<crate::io::RawOutputSwitch>>,
           wl_rx: Arc<GenericReceiver<crate::acm::WhitelistAccessRequest>>)
{
    loop
    {
        let queue = select_chan!(io_rx, wl_rx);
        match queue
        {
            0 => {
                let res = io_rx.receive();
                sender.send(format!("io_request: {:?} {:?}", res.output_id, res.target_state)).unwrap();
                },
            1 => {
                let res = wl_rx.receive();
                sender.send(format!("access point: {:?}", res.identity_token_number)).unwrap();
                },
            _ => {}
        }
    }
}

struct Server
{
    server: Sender,
    io_events_rx: Arc<GenericReceiver<crate::io::RawOutputSwitch>>,
    wl_events_rx: Arc<GenericReceiver<crate::acm::WhitelistAccessRequest>>,
}

impl Handler for Server
{
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        let sender = self.server.clone();
        let rx = self.io_events_rx.clone();
        let wl_rx = self.wl_events_rx.clone();
        thread::spawn(move || {
            send_io(sender, rx, wl_rx);
        });

        Ok(())
    }
    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Server gets message: {:?}", msg);
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("connection closed: {:?}", code);
        self.server.shutdown().unwrap();
    }

    // and more ...
}

struct Websocket 
{
    tracer: trace_helper::TraceHelper,
    door_rx: Arc<GenericReceiver<crate::dcm::DoorOpenRequest>>,
    raw_output_switch: Arc<GenericReceiver<crate::io::RawOutputSwitch>>,
    wl_rx: Arc<GenericReceiver<crate::acm::WhitelistAccessRequest>>,
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
            raw_output_switch: chm.get_receiver::<crate::io::RawOutputSwitch>(),
            wl_rx: chm.get_receiver::<crate::acm::WhitelistAccessRequest>(),
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
        let rx_clone = Arc::clone(&self.door_rx);
        listen(ADDR, |out| Server {
            server: out,
            io_events_rx: self.raw_output_switch.clone(),
            wl_events_rx: self.wl_rx.clone()
        }).unwrap();
        true
    }

    pub fn work(&self)
    {
        self.init();
        self.run();
    }

}