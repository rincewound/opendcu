use rouille::*;

use crate::core::broadcast_channel::*;
use crate::core::{channel_manager::*};
use crate::trace::*;
use std::{sync::{Arc}, thread};
use std::io::Read;
use crate::core::{shareable::Shareable, bootstage_helper::*};
use crate::cfg::cfgholder::*;


const MODULE_ID: u32 = 0x06000000;

pub fn launch(chm: &mut ChannelManager)
{        
    let tracer = trace_helper::TraceHelper::new("CFG/Rest".to_string(), chm);
    let mut cr = ConfigRest::new(tracer, chm);      
    thread::spawn(move|| {
        cr.start();
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
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    cfg_publish_tx: GenericSender<crate::cfg::ConfigMessage>,
    cfg: Shareable<crate::cfg::cfgholder::CfgHolder>,
    
}

impl ConfigRest //<'a>
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
        let theSender = self.cfg_publish_tx.clone();
        let theCfg = self.cfg.clone();
        let cbs = [None, Some(move|| {
            theSender.send(super::ConfigMessage::RegisterHandlers(theCfg))
        })];

        boot(MODULE_ID, cbs, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);
    }

    fn do_put(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        let mut reqdata = Vec::new();
        let mut d = req.data().unwrap();
        d.read_to_end(&mut reqdata);
        self.cfg.lock()
                .unwrap()
                .do_put(_module, reqdata);
        rouille::Response::text("All is bad.")
    }

    fn do_post(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        print!("{}", req.url());
        rouille::Response::text("All is bad.")
    }

    fn do_get(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        print!("{}", req.url());
        rouille::Response::text("All is bad.")
    }

    fn do_delete(&self, req: &rouille::Request, _module: String) -> rouille::Response
    {
        print!("{}", req.url());
        rouille::Response::text("All is bad.")
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
                self.do_put(&request, module)
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

            // The code block is called if none of the other blocks matches the request.
            // We return an empty response with a 404 status code.
            _ => rouille::Response::empty_404()
        )
    });    
    }      

    // to simplify the call inside the launch function
    // which depend on the which is used to be started
    pub fn start(mut self)
    {
        self.init();
        self.run();
    }
}

#[cfg(test)]
mod tests {
     
}