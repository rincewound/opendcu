
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering};

use crate::util::{JsonStorage, ObjectStorage};


#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum Weekday
{
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TimeSlot
{
    pub day: Weekday,
    pub from: u16,
    pub to: u16
}

// pub enum AccessFlags
// {
//     Access,
//     Pin
// }

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct AccessProfile
{
    pub id: u16,
    // All access points, for which this profile is valid
    pub access_points: Vec<u32>,
    pub time_pro: Vec<TimeSlot>
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct WhitelistEntry
{
    pub identification_token_id: Vec<u8>,
    pub access_profiles: Vec<u16> // Note: This should be a ref to another table or similar
}

pub trait WhitelistEntryProvider
{
    fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<WhitelistEntry>;
    fn put_entry(&mut self,entry: WhitelistEntry);
    fn delete_entry(&mut self, identity_token_id: Vec<u8>);
    fn new() -> Self;
}

pub trait ProfileChecker
{
    fn check_profile(&self, ap_id: u32, entry: &WhitelistEntry) -> bool;
    fn add_profile(&mut self, profile: AccessProfile);
    fn get_profile(&self, profile_id_: u32) -> Option<AccessProfile>;
    fn delete_profile(&mut self, profile_id: u32);
}

pub struct JsonProfileChecker{
    profiles: JsonStorage<AccessProfile>
}

impl ProfileChecker for JsonProfileChecker
{
    fn check_profile(&self, ap_id: u32, entry: &WhitelistEntry) -> bool 
    {
        for profile_id in entry.access_profiles.iter()
        {
            let profile = self.profiles.get_entry(|x| x.id == *profile_id);
            if let Some(the_profile) = profile
            {   
                if the_profile.access_points.iter().find(|&&x| x == ap_id).is_none() { break; }
                
                // the ap is contained in the profile. Check the time_pro
                // and be done with it.
                for tp in the_profile.time_pro.iter()
                {
                    if tp.is_active()
                    {
                        return true;
                    }
                }
                
            }
        }
        return false;
    }
    fn add_profile(&mut self, profile: AccessProfile) {
        self.profiles.delete_entry(|x| x.id == profile.id as u16);
        self.profiles.put_entry(profile);
        self.profiles.update_storage();
    }

    fn get_profile(&self, profile_id: u32) -> Option<AccessProfile> {
        self.profiles.get_entry(|x| x.id == profile_id as u16)
    }
    
    fn delete_profile(&mut self, profile_id: u32) {
        self.profiles.delete_entry(|x| x.id == profile_id as u16);
        self.profiles.update_storage();
    }
    
}

///   # The JsonEntry Provider
///   This is a decidedly simple way of storing whitelist
///   entries. It will just serialize the entry to a plain
///   textfile
///
///   This is fine for development purposes and very
///   small whitelists. However:
///   * The whole whitelist is kept in RAM at all times
///   * The whole whitelist is serialized and written to
///     storage for each change
/// 
///  This behavior makes the implementation suboptimal
///  for all but the simplest and most static production
///  uses.
 pub struct JsonEntryProvider
{
    entries: JsonStorage<WhitelistEntry>
}

impl WhitelistEntryProvider for JsonEntryProvider
{    
    fn new() -> Self
    {
        return JsonEntryProvider
        {
            entries: JsonStorage::new("whitelist.txt".to_string())
        }
    }
    
    fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<WhitelistEntry> 
    { 
        return self.entries.get_entry(|x| x.identification_token_id.cmp(&identity_token_id) == Ordering::Equal);
    }

    fn put_entry(&mut self, entry: WhitelistEntry) 
    { 
        // delete entry if already existing..
        self.entries.delete_entry(|x| x.identification_token_id.cmp(&entry.identification_token_id) == Ordering::Equal);
        self.entries.put_entry(entry);
        self.entries.update_storage();
    }

    fn delete_entry(&mut self, identity_token_id: Vec<u8>) 
    { 
        self.entries.delete_entry(|x| x.identification_token_id.cmp(&identity_token_id) != Ordering::Equal);
        self.entries.update_storage();
    }
}

impl TimeSlot
{
    pub fn is_active(&self) -> bool
    {
        return true;
    }
}
