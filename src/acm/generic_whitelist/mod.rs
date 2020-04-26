

use crate::core::BootStage;
use crate::core::BroadcastChannel::*;
use crate::core::{SystemMessage, ChannelManager::*};
use crate::Trace;
use crate::{sig::*, acm::*};
use std::{sync::Arc, thread};
use crate::cfg;

mod whitelist;

const Module_ID: u32 = 0x03000000;

pub fn launch(chm: &mut ChannelManager)
{    
    let tracer = Trace::TraceHelper::TraceHelper::new("ACM/Whitelist".to_string(), chm);
    let mut wl = GenericWhitelist::new(tracer, chm, whitelist::SqliteEntryProvider);
    thread::spawn(move || {  
        wl.init();   
        loop 
        {
            if !wl.do_request()
            {
                break;
            }
        }   
        
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
      // deserialize data from req
      add_whitelist(deserialized data)
  }

*/


struct GenericWhitelist<WhitelistProvider: whitelist::WhitelistEntryProvider>
{
    tracer: Trace::TraceHelper::TraceHelper,
    access_request_rx: Arc<GenericReceiver<crate::acm::WhitelistAccessRequest>>,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    sig_tx: GenericSender<crate::sig::SigCommand>,
    door_tx: GenericSender<crate::dcm::DoorOpenRequest>,
    whitelist: WhitelistProvider
}

impl<WhitelistProvider: whitelist::WhitelistEntryProvider> GenericWhitelist<WhitelistProvider>
{
    fn new(trace: Trace::TraceHelper::TraceHelper, chm: &mut ChannelManager, whitelist: WhitelistProvider) -> Self
    {
        GenericWhitelist
        {
            tracer: trace,
            access_request_rx: chm.get_receiver::<crate::acm::WhitelistAccessRequest>(),
            system_events_rx: chm.get_receiver::<crate::core::SystemMessage>(),
            system_events_tx: chm.get_sender::<crate::core::SystemMessage>(),
            sig_tx: chm.get_sender::<crate::sig::SigCommand>(),
            door_tx: chm.get_sender::<crate::dcm::DoorOpenRequest>(),
            whitelist
        }
    }

    pub fn init(&mut self)
    {
        crate::core::BootstageHelper::Boot(Module_ID, self.system_events_tx.clone(), self.system_events_rx.clone(), &self.tracer);        
    }

    pub fn do_request(&mut self) -> bool
    {
        self.tracer.TraceStr("Start serving requests.");
        let req = self.access_request_rx.receive();
        self.tracer.TraceStr("Received request.");
        // ToDo: This should be done from a threadpool.
        self.process_access_request(req);
        true
    }

    fn send_signal_command(&self, access_point_id: u32, sigtype: SigType, duration: u32)
    {
        let sig = SigCommand {
            access_point_id: access_point_id,
            sig_type: sigtype, 
            duration: duration
        };

        self.sig_tx.send(sig); 
    }

    fn process_access_request(&self, req: WhitelistAccessRequest)
    {
        // Pull Whitelist Entry
        let entry = self.whitelist.get_entry(req.identity_token_number);

        // Found? If so, check access profile, otherwise emit AccessDenied Sig
        if let Some(entry) = entry 
        {
            
            // Good? If so, emit DoorOpenRequest, otherwise emit AccessDenied Sig 
            let openreq = crate::dcm::DoorOpenRequest {access_point_id: req.access_point_id};
            self.door_tx.send(openreq);
        }
        else
        {
            self.tracer.TraceStr("Access Denied; Unknown identifiaction token.");
            self.send_signal_command(req.access_point_id, SigType::AccessDenied, 1000);
        }
    }


    
    fn process_put_req(entry: whitelist::WhitelistEntry)
    {

    }

    fn register_handlers()
    {
        let h = Handler!(GenericWhitelist::<WhitelistProvider>::process_put_req);
    }

}

#[cfg(test)]
mod tests {
     use crate::{core::ChannelManager::ChannelManager, acm::*, Trace, sig::SigCommand};
     use generic_whitelist::whitelist::WhitelistEntry;
     use crate::sig::*;

     struct DummyWhitelist
     {
        pub entry: Option<WhitelistEntry>
     }

     impl DummyWhitelist
     {
         pub fn new() -> Self
         {
             DummyWhitelist{entry: None}
         }
     }

     impl crate::acm::generic_whitelist::whitelist::WhitelistEntryProvider for DummyWhitelist
     {
         fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<generic_whitelist::whitelist::WhitelistEntry> 
         { 
             self.entry.clone()
         }
         fn put_entry(&self,entry: generic_whitelist::whitelist::WhitelistEntry) { unimplemented!() }

     }  

     #[test]
     fn will_throw_access_denied_if_no_whitelist_entry_exists()
     {
         let mut chm = ChannelManager::new();
         let wl = DummyWhitelist::new();
         let tracer = Trace::TraceHelper::TraceHelper::new("ACM/Whitelist".to_string(), &mut chm);
         let mut md = generic_whitelist::GenericWhitelist::new(tracer, &mut chm, wl);

         let sig_rx = chm.get_receiver::<SigCommand>();
         let access_tx = chm.get_sender::<WhitelistAccessRequest>();

         let req = WhitelistAccessRequest {
             access_point_id: 0,
             identity_token_number: vec![1,2,3,4],
         };

         access_tx.send(req);
         md.do_request();
         let res = sig_rx.receive_with_timeout(1);
         if let Some(x) = res {
            assert!(x.sig_type == SigType::AccessDenied)
         }
         else
         {
             assert!(false)
         }
     }

     #[test]
     fn will_throw_access_denied_if_no_access_rights()
     {
        assert_eq!(true, false)
     }

     #[test]
     fn will_generate_door_open_request_if_access_rights_are_good()
     {
        assert_eq!(true, false)
     }

     
     #[test]
     fn will_generate_door_open_request_if_token_is_known()
     {
        let mut chm = ChannelManager::new();
        let mut wl = DummyWhitelist::new();
        wl.entry = Some(WhitelistEntry{
            access_token_id: Vec::new(),
            access_profiles: Vec::new()

        });
        let tracer = Trace::TraceHelper::TraceHelper::new("ACM/Whitelist".to_string(), &mut chm);
        let mut md = generic_whitelist::GenericWhitelist::new(tracer, &mut chm, wl);

        let dcm_rx = chm.get_receiver::<crate::dcm::DoorOpenRequest>();
        let access_tx = chm.get_sender::<WhitelistAccessRequest>();

        let req = WhitelistAccessRequest {
            access_point_id: 47,
            identity_token_number: vec![1,2,3,4],
        };

        access_tx.send(req);
        md.do_request();
        let res = dcm_rx.receive_with_timeout(1);
        if let Some(x) = res {
           assert!(x.access_point_id == 47)
        }
        else
        {
            assert!(false)
        }
     }
}