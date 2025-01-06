use barracuda_core::{core::{SystemMessage, bootstage_helper::{boot, boot_noop}, broadcast_channel::{GenericReceiver, GenericSender}, channel_manager::ChannelManager, shareable::Shareable}, trace::trace_helper};
use rouille::*;



use std::{thread};
use std::io::Read;

use crate::cfg::cfgholder::*;


const MODULE_ID: u32 = 0x06000000;

pub fn launch(chm: &mut ChannelManager)
{        
    let tracer = trace_helper::TraceHelper::new("CFG/Rest".to_string(), chm);
    let mut cr = ConfigRest::new(tracer, chm);      
    thread::spawn(move|| {
        cr.init();  
        cr.run();         
    });
}

/*
Idea for configuration:
* Each module should have its own cfg channel where cfg changes can be injected. This would
* allow us to use free functions for actually injecting the data. ideally the API would be
* something like:

#[Post, Endpoint=ACM/Whitelist/{InstanceId}/]
fn add_whitelist(wlentry: WhitelistEntry, InstanceId: u32) -> Success
{

}

However: 
* We need access to the channel manager for this to work
* There needs to be a way to gather all endpoints across all active services
  (ideally during LLI)


  -> The Macro Expasion of #Post would be something like
  fn _post_add_whitelist(req: Request) -> Success
  {
      // deserialize data from req into correct type
      add_whitelist(deserialized data)
  }

*/


struct ConfigRest
{
    tracer: trace_helper::TraceHelper,
    system_events_rx: GenericReceiver<SystemMessage>,
    system_events_tx: GenericSender<SystemMessage>,
    cfg_publish_tx: GenericSender<crate::cfg::ConfigMessage>,
    cfg: Shareable<crate::cfg::cfgholder::CfgHolder>,
    
}

impl ConfigRest
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager) -> Self
    {
        ConfigRest
        {
            tracer: trace,            
            system_events_rx: chm.get_receiver(),
            system_events_tx: chm.get_sender(),
            cfg_publish_tx: chm.get_sender(),
            cfg: Shareable::new(CfgHolder::new())
        }

    }

    pub fn init(&mut self)
    {
        let the_sender = self.cfg_publish_tx.clone();
        let the_cfg = self.cfg.clone();
        let hlicb = Some(move|| {
            the_sender.send(super::ConfigMessage::RegisterHandlers(the_cfg))
        });

        boot(MODULE_ID, Some(boot_noop), hlicb, 
            &self.system_events_tx, 
            &self.system_events_rx, 
            &self.tracer);
    }    

    fn do_put(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        let mut reqdata = Vec::new();
        let mut d = req.data().unwrap();
        let _ = d.read_to_end(&mut reqdata);
        self.cfg.lock()
                .do_put(_module, reqdata);
        rouille::Response::text("ok").with_status_code(200)
    }

    fn do_post(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        print!("{}", req.url());
        let mut reqdata = Vec::new();
        let mut d = req.data().unwrap();
        let _ = d.read_to_end(&mut reqdata);
        self.cfg.lock()
                .do_post(_module, reqdata);        
        rouille::Response::text("ok").with_status_code(200)
    }

    fn do_get(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        print!("{}", req.url());
        let mut reqdata = Vec::new();
        let mut d = req.data().unwrap();
        let _ = d.read_to_end(&mut reqdata);
        let response = self.cfg.lock()
                              .do_get(_module);
        match response
        {            
            Ok(data) => rouille::Response::from_data("application/octet-stream", data).with_status_code(200),   // Ok
            Err(CfgError::ResourceEmpty) => rouille::Response::text("Resource empty").with_status_code(406),                       // Not Acceptable
            Err(CfgError::ResourceNotFound) => rouille::Response::text("Resource not found").with_status_code(404)                 // Not found 
        }
    }

    fn do_delete(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        print!("{}", req.url());
        let mut reqdata = Vec::new();
        let mut d = req.data().unwrap();
        let _ = d.read_to_end(&mut reqdata);
        self.cfg.lock()
                .do_delete(_module, reqdata);
        rouille::Response::text("ok").with_status_code(200)
    }

    pub fn run(self) -> bool
    {
        // The `start_server` starts listening forever on the given address.
        rouille::start_server("localhost:8000", move |request| {        
        router!(request,
            (GET) (/) => {
                // If the request's URL is `/`, we jump here.
                // This block builds a `Response` object that redirects to the `/hello/world`.
                rouille::Response::text("barracuda configuration interface")
            },

            (PUT) (/api/{module: String}) => {
                self.do_put(&request, module)
            },

            (PUT) (/api/{module: String}/{submodule: String}) => {
                self.do_put(&request, format!("{}/{}", module, submodule))
            },

            (POST) (/api/{module: String}) => {
                self.do_post(&request, module)
            },

            (GET) (/api/{module: String}) => {
                self.do_get(&request, module)
            },

            (DELETE) (/api/{module: String}) => {
                self.do_delete(&request, module)
            },

            (DELETE) (/api/{module: String}/{submodule: String}) => {
                self.do_delete(&request, format!("{}/{}", module, submodule))
            },

            // The code block is called if none of the other blocks matches the request.
            // We return an empty response with a 404 status code.
            _ => rouille::Response::empty_404()
        )
    });    
    }      
}

#[cfg(test)]
mod tests {
     
}