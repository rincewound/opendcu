
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering};

use crate::util::json_storage;
use crate::util::ObjectStorage;


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
    entries: json_storage<WhitelistEntry>
}

impl WhitelistEntryProvider for JsonEntryProvider
{    
    fn new() -> Self
    {
        return JsonEntryProvider
        {
            entries: json_storage::new("whitelist.txt".to_string())
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
