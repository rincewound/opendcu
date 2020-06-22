extern crate barracuda_core;
extern crate barracuda_hal;

use barracuda_core::core::broadcast_channel::*;
use barracuda_core::core::channel_manager::*;
use barracuda_core::core::{shareable::Shareable, bootstage_helper::*, SystemMessage};
use barracuda_core::cfg;
use barracuda_core::cfg::cfgholder::*;
use barracuda_core::trace::*;
use barracuda_core::{sig::*, acm::*};
use barracuda_core::dcm::DoorOpenRequest;
use std::{sync::Arc, thread};

use cfg::ConfigMessage;
use profiles::{ProfileChecker, JsonProfileChecker, AccessProfile};

pub mod whitelist;

mod profiles;

const MODULE_ID: u32 = 0x03000000;

pub fn launch<T: 'static>(chm: &mut ChannelManager)
    where T: whitelist::WhitelistEntryProvider + std::marker::Send
{    
    let tracer = trace_helper::TraceHelper::new("ACM/Whitelist".to_string(), chm);
    let mut wl = GenericWhitelist::new(tracer, chm, T::new(), JsonProfileChecker::new("profiles.txt".to_string()));
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


struct GenericWhitelist<WhitelistProvider: whitelist::WhitelistEntryProvider, ProfileStorage: ProfileChecker>
{
    tracer              : trace_helper::TraceHelper,
    access_request_rx   : Arc<GenericReceiver<WhitelistAccessRequest>>,
    cfg_rx              : Arc<GenericReceiver<ConfigMessage>>,
    system_events_rx    : Arc<GenericReceiver<SystemMessage>>,
    system_events_tx    : GenericSender<SystemMessage>,
    sig_tx              : GenericSender<SigCommand>,
    door_tx             : GenericSender<DoorOpenRequest>,
    whitelist           : Shareable<WhitelistProvider>,
    profiles            : Shareable<ProfileStorage>    
}

impl<WhitelistProvider: whitelist::WhitelistEntryProvider + Send + 'static, ProfileStorage:ProfileChecker + Send +'static> GenericWhitelist<WhitelistProvider, ProfileStorage>
{
    fn new(trace: trace_helper::TraceHelper, chm: &mut ChannelManager, whitelist: WhitelistProvider, profile_source: ProfileStorage) -> Self
    { 
        GenericWhitelist
        {
            tracer              : trace,
            access_request_rx   : chm.get_receiver(),
            cfg_rx              : chm.get_receiver(), 
            system_events_rx    : chm.get_receiver(),
            system_events_tx    : chm.get_sender(),
            sig_tx              : chm.get_sender(),
            door_tx             : chm.get_sender(),
            whitelist           : Shareable::new(whitelist),
            profiles            : Shareable::new(profile_source)
        }
    }

    pub fn init(&mut self)
    {    
        let the_receiver = self.cfg_rx.clone();  
        let hli_cb= Some(|| {
            /*
                This is executed during HLI
            */

            let res = the_receiver.receive();
            let crate::cfg::ConfigMessage::RegisterHandlers(cfg_holder) = res;
            let mut holder = cfg_holder.lock();
            let wl1 = self.whitelist.clone();
            let wl2 = self.whitelist.clone();

            let prof1 = self.profiles.clone();
            let prof2 = self.profiles.clone();

            holder.register_handler(FunctionType::Put, "wl/entry".to_string(), Handler!(|r: whitelist::WhitelistEntry|
                {
                    Self::process_put_entry_req(&wl1, r);
                }));

            holder.register_handler(FunctionType::Delete, "wl/entry".to_string(), Handler!(|r: whitelist::WhitelistEntry|
                {
                    Self::process_delete_entry_req(&wl2, r);
                }));

            holder.register_handler(FunctionType::Put, "wl/profile".to_string(), Handler!(|newprofile: AccessProfile|
                {
                    Self::process_put_profile_req(&prof1, newprofile)
                }));
            
            holder.register_handler(FunctionType::Delete, "wl/profile".to_string(), Handler!(|profile_to_delete: AccessProfile|
                {
                    Self::process_delete_profile_req(&prof2, profile_to_delete);
                    
                }))
            
        });

        boot(MODULE_ID, Some(boot_noop), hli_cb, 
            self.system_events_tx.clone(), 
            self.system_events_rx.clone(), 
            &self.tracer);

    }

    pub fn do_request(&mut self) -> bool
    {
        self.tracer.trace_str("Start serving requests.");
        let req = self.access_request_rx.receive();
        self.tracer.trace(format!("Received request with token {:?}", req.identity_token_number));
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

    fn check_profile(&self, ap_id: u32, entry: &whitelist::WhitelistEntry) -> bool
    {
        let profile_result = self.profiles.lock().check_profile(ap_id, entry);
        match profile_result
        {
            Ok(_) => {
                return true;
            },
            Err(reason) =>
            {
                self.tracer.trace(format!("Access Denied for AP {}, Reason: {}",ap_id, reason));
                self.send_signal_command(ap_id as u32, SigType::AccessDenied, 1000);                   
                return false;             
            }
        }
    }

    fn process_access_request(&self, req: WhitelistAccessRequest)
    {
        // Pull Whitelist Entry
        let entry = self.whitelist.lock().get_entry(req.identity_token_number);

        // Found? If so, check access profile, otherwise emit AccessDenied Sig
        if let Some(entry) = entry 
        {
            if !self.check_profile(req.access_point_id, &entry) { return; }

            // Good? If so, emit DoorOpenRequest, otherwise emit AccessDenied Sig 
            self.tracer.trace(format!("Request seems ok for token {:?}, sending door open request.", entry.identification_token_id));
            let openreq = barracuda_core::dcm::DoorOpenRequest {access_point_id: req.access_point_id};
            self.door_tx.send(openreq);
               
        }
        else
        {
            self.tracer.trace_str("Access Denied; Unknown identification token.");
            self.send_signal_command(req.access_point_id, SigType::AccessDenied, 1000);
        }
    }


    fn process_put_entry_req(wl: &Shareable<WhitelistProvider>, entry: whitelist::WhitelistEntry)
    {
        let mut thewhitelist = wl.lock();
        thewhitelist.put_entry(entry);
    }

    fn process_delete_entry_req(wl: &Shareable<WhitelistProvider>, entry: whitelist::WhitelistEntry)
    {
        let mut thewhitelist = wl.lock();
        thewhitelist.delete_entry(entry.identification_token_id);
    }

    fn process_put_profile_req(current_profiles: &Shareable<ProfileStorage>, profile: AccessProfile)
    {
        let json = serde_json::to_string(&profile).unwrap();        
        println!("Add profile: {}", json);
        let mut profiles = current_profiles.lock();        
        // remove any existing profile with the same id:
        profiles.delete_profile(profile.id as u32);
        profiles.add_profile(profile);        
    }

    fn process_delete_profile_req(current_profiles: &Shareable<ProfileStorage>, profile: AccessProfile)
    {        
        println!("Delete profile with id: {}", profile.id);
        let mut profiles = current_profiles.lock();
        
        // remove any existing profile with the same id:
        profiles.delete_profile(profile.id as u32 );
    }

}



#[cfg(test)]
mod tests {
     use barracuda_core::{core::channel_manager::ChannelManager, acm::*, trace::*, sig::SigCommand};
     use crate::profiles::{AccessProfile, ProfileChecker, ProfileCheckResult};
     use crate::whitelist::WhitelistEntry;
     use crate::whitelist::WhitelistEntryProvider;
     use barracuda_core::{sig::*};

     struct DummyWhitelist
     {
        pub entry: Option<WhitelistEntry>
     }

     impl crate::whitelist::WhitelistEntryProvider for DummyWhitelist
     {         
        fn new() -> Self
        {
            DummyWhitelist{entry: None}
        }

         fn get_entry(&self, _identity_token_id: Vec<u8>) -> Option<crate::whitelist::WhitelistEntry> 
         { 
             self.entry.clone()
         }
         fn put_entry(&mut self, entry: crate::whitelist::WhitelistEntry) 
         { 
            self.entry = Some(entry);
         }
         fn delete_entry(&mut self, _identity_token_id: Vec<u8>) { 
             self.entry = None;
         }

     }  

     struct DummyProfileChecker
     {
         pub check_result: Result<(), ProfileCheckResult>
     }

     impl ProfileChecker for DummyProfileChecker
     {
        fn check_profile(&self, _ap_id: u32, _entry: &WhitelistEntry)  -> Result<(), ProfileCheckResult>
        {
            return self.check_result;
        }

        fn add_profile(&mut self, _profile: AccessProfile) {}
        fn get_profile(&self, _profile_id_: u32) -> Option<AccessProfile> {None}
        fn delete_profile(&mut self, _profile_id: u32) { }         
     }

     fn make_whitelist(chm: &mut ChannelManager) -> crate::GenericWhitelist<DummyWhitelist, DummyProfileChecker>
     {
        let wl = DummyWhitelist::new();
        let prof = DummyProfileChecker {check_result: Ok(())};
        let tracer = trace_helper::TraceHelper::new("ACM/Whitelist".to_string(), chm);
        let md = crate::GenericWhitelist::new(tracer, chm, wl, prof);
        return md;
     }

     #[test]
     fn will_throw_access_denied_if_no_whitelist_entry_exists()
     {
         let mut chm = ChannelManager::new();
         let mut wl = make_whitelist(&mut chm);

         let sig_rx = chm.get_receiver::<SigCommand>();
         let access_tx = chm.get_sender::<WhitelistAccessRequest>();

         let req = WhitelistAccessRequest {
             access_point_id: 0,
             identity_token_number: vec![1,2,3,4],
         };

         access_tx.send(req);
         wl.do_request();
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
     fn will_generate_door_open_request_if_token_is_known_and_access_rights_are_good()
     {
        let mut chm = ChannelManager::new();
        let mut wl = DummyWhitelist::new();
        wl.entry = Some(WhitelistEntry{
            identification_token_id: Vec::new(),
            access_profiles: vec![1]

        });
        let tracer = trace_helper::TraceHelper::new("ACM/Whitelist".to_string(), &mut chm);
        let mut md = crate::GenericWhitelist::new(tracer, &mut chm, wl, DummyProfileChecker {check_result: Ok(())});

        let dcm_rx = chm.get_receiver::<barracuda_core::dcm::DoorOpenRequest>();
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

     #[test]
     fn generate_multiple_door_open_requests()
     {
        let mut chm = ChannelManager::new();
        let mut wl = DummyWhitelist::new();
        wl.entry = Some(WhitelistEntry{
            identification_token_id: Vec::new(),
            access_profiles: Vec::new()

        });
        let tracer = trace_helper::TraceHelper::new("ACM/Whitelist".to_string(), &mut chm);
        let mut md = crate::GenericWhitelist::new(tracer, &mut chm, wl,DummyProfileChecker {check_result: Ok(())});

        let dcm_rx = chm.get_receiver::<barracuda_core::dcm::DoorOpenRequest>();
        let access_tx = chm.get_sender::<WhitelistAccessRequest>();

        for _ in 0..20
        {
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
}