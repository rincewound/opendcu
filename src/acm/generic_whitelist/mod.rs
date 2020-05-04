use crate::core::broadcast_channel::*;
use crate::core::{channel_manager::*};
use crate::trace::*;
use crate::{sig::*, acm::*};
use std::{sync::{Mutex, Arc}, thread};
use crate::cfg;
use crate::cfg::cfgholder::*;
use crate::core::bootstage_helper::*;

pub mod whitelist;

const MODULE_ID: u32 = 0x03000000;

pub fn launch<T: 'static>(chm: &mut ChannelManager)
    where T: whitelist::WhitelistEntryProvider + std::marker::Send
{    
    let tracer = trace_helper::TraceHelper::new("ACM/Whitelist".to_string(), chm);
    let mut wl = GenericWhitelist::new(tracer, chm, T::new());
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


struct GenericWhitelist<WhitelistProvider: whitelist::WhitelistEntryProvider>
{
    tracer: trace_helper::TraceHelper,
    access_request_rx: Arc<GenericReceiver<crate::acm::WhitelistAccessRequest>>,
    cfg_rx: Arc<GenericReceiver<crate::cfg::ConfigMessage>>,
    system_events_rx: Arc<GenericReceiver<crate::core::SystemMessage>>,
    system_events_tx: GenericSender<crate::core::SystemMessage>,
    sig_tx: GenericSender<crate::sig::SigCommand>,
    door_tx: GenericSender<crate::dcm::DoorOpenRequest>,
    whitelist: Arc<Mutex<WhitelistProvider>>
}

impl<WhitelistProvider: whitelist::WhitelistEntryProvider + Send + 'static> GenericWhitelist<WhitelistProvider>
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager, whitelist: WhitelistProvider) -> Self
    {
        GenericWhitelist
        {
            tracer              : trace,
            access_request_rx   : chm.get_receiver::<crate::acm::WhitelistAccessRequest>(),
            cfg_rx              : chm.get_receiver::<crate::cfg::ConfigMessage>(), 
            system_events_rx    : chm.get_receiver::<crate::core::SystemMessage>(),
            system_events_tx    : chm.get_sender::<crate::core::SystemMessage>(),
            sig_tx              : chm.get_sender::<crate::sig::SigCommand>(),
            door_tx             : chm.get_sender::<crate::dcm::DoorOpenRequest>(),
            whitelist           : Arc::new(Mutex::new(whitelist))
        }
    }

    pub fn init(&mut self)
    {    
        let the_receiver = self.cfg_rx.clone();  
        let the_whitelist = self.whitelist.clone();
        let cbs= [None, Some(move|| {
            /*
                This is executed during HLI
            */

            let res = the_receiver.receive();
            let crate::cfg::ConfigMessage::RegisterHandlers(cfg_holder) = res;
            let mut holder = cfg_holder.lock().unwrap();
            let wl1 = the_whitelist.clone();
            let wl2 = the_whitelist.clone();

            holder.register_handler(FunctionType::Put, "wl".to_string(), Handler!(|r: whitelist::WhitelistEntry|
                {
                    GenericWhitelist::<WhitelistProvider>::process_put_req(wl1.clone(), r);
                }));

            holder.register_handler(FunctionType::Delete, "wl".to_string(), Handler!(|r: whitelist::WhitelistEntry|
                {
                    GenericWhitelist::<WhitelistProvider>::process_delete_req(wl2.clone(), r);
                }))
            
        })];

        boot(MODULE_ID, cbs, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);
    }

    pub fn do_request(&mut self) -> bool
    {
        self.tracer.trace_str("Start serving requests.");
        let req = self.access_request_rx.receive();
        self.tracer.trace_str("Received request.");
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
        let entry = self.whitelist.lock()
                                                          .unwrap()
                                                          .get_entry(req.identity_token_number);

        // Found? If so, check access profile, otherwise emit AccessDenied Sig
        if let Some(entry) = entry 
        {
            
            // Good? If so, emit DoorOpenRequest, otherwise emit AccessDenied Sig 
            self.tracer.trace(format!("Request seems ok for token {:?}, sending door open request.", entry.access_token_id));
            let openreq = crate::dcm::DoorOpenRequest {access_point_id: req.access_point_id};
            self.door_tx.send(openreq);
        }
        else
        {
            self.tracer.trace_str("Access Denied; Unknown identifiaction token.");
            self.send_signal_command(req.access_point_id, SigType::AccessDenied, 1000);
        }
    }


    fn process_put_req(wl: Arc<Mutex<WhitelistProvider>>, entry: whitelist::WhitelistEntry)
    {
        println!("PUT into whitelist.");
        let mut thewhitelist = wl.lock().unwrap();
        thewhitelist.put_entry(entry);
    }

    fn process_delete_req(wl: Arc<Mutex<WhitelistProvider>>, entry: whitelist::WhitelistEntry)
    {
        println!("DELETE from whitelist.");
        let mut thewhitelist = wl.lock().unwrap();
        thewhitelist.delete_entry(entry.access_token_id);
    }

}

#[cfg(test)]
mod tests {
     use crate::{core::channel_manager::ChannelManager, acm::*, trace::*, sig::SigCommand};
     use generic_whitelist::whitelist::WhitelistEntry;
     use crate::acm::generic_whitelist::whitelist::WhitelistEntryProvider;
     use crate::sig::*;

     struct DummyWhitelist
     {
        pub entry: Option<WhitelistEntry>
     }

     impl crate::acm::generic_whitelist::whitelist::WhitelistEntryProvider for DummyWhitelist
     {         
        fn new() -> Self
        {
            DummyWhitelist{entry: None}
        }

         fn get_entry(&self, _identity_token_id: Vec<u8>) -> Option<generic_whitelist::whitelist::WhitelistEntry> 
         { 
             self.entry.clone()
         }
         fn put_entry(&mut self, entry: generic_whitelist::whitelist::WhitelistEntry) 
         { 
            self.entry = Some(entry);
         }
         fn delete_entry(&mut self, _identity_token_id: Vec<u8>) { 
             self.entry = None;
         }

     }  

     #[test]
     fn will_throw_access_denied_if_no_whitelist_entry_exists()
     {
         let mut chm = ChannelManager::new();
         let wl = DummyWhitelist::new();
         let tracer = trace_helper::TraceHelper::new("ACM/Whitelist".to_string(), &mut chm);
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
            //access_profiles: Vec::new()

        });
        let tracer = trace_helper::TraceHelper::new("ACM/Whitelist".to_string(), &mut chm);
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