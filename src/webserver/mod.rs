use crate::{trace::trace_helper, core::{broadcast_channel::{GenericSender, GenericReceiver}, channel_manager::ChannelManager}};
use std::{thread, sync::Arc};
use rouille::Response;

const MODULE_ID: u32 = 0x05000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = trace_helper::TraceHelper::new("WWW/cfgif".to_string(), chm);
    let mut wl = Webserver::new(tracer, chm);
    thread::spawn(move || {  
        wl.init();
        wl.run();           
    });
}


struct Webserver
{
    tracer: trace_helper::TraceHelper,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
}

impl Webserver
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        Webserver
        {
            tracer: trace,            
            system_events_rx: chm.get_receiver::<crate::core::SystemMessage>(),
            system_events_tx: chm.get_sender::<crate::core::SystemMessage>(),  
        }
    }

    pub fn init(&mut self)
    {
        crate::core::bootstage_helper::plain_boot(MODULE_ID, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);
    }

    pub fn run(self) -> bool
    {
        
    rouille::start_server("localhost:8001", move |request| {
        {
            // The `match_assets` function tries to find a file whose name corresponds to the URL
            // of the request. The second parameter (`"."`) tells where the files to look for are
            // located.
            // In order to avoid potential security threats, `match_assets` will never return any
            // file outside of this directory even if the URL is for example `/../../foo.txt`.
            let response = rouille::match_assets(&request, "src/ui");

            // If a file is found, the `match_assets` function will return a response with a 200
            // status code and the content of the file. If no file is found, it will instead return
            // an empty 404 response.
            // Here we check whether if a file is found, and if so we return the response.
            if response.is_success() {
                return response;
            }
        }

        // This point of the code is reached only if no static file matched the request URL.

        // In a real website you probably want to serve non-static files here (with the `router!`
        // macro for example), but here we just return a 404 response.
        Response::html("404 error. Try <a href=\"/README.md\"`>README.md</a> or \
                        <a href=\"/src/lib.rs\">src/lib.rs</a> for example.")
            .with_status_code(404)
        });
    }      
}