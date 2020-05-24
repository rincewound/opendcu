
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fs::File};

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
    //pub access_profiles: Vec<u16> // Note: This should be a ref to another table or similar
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
    entries: Vec<WhitelistEntry>
}

impl JsonEntryProvider
{
    fn update_storage(&self)
    {
        let writer = File::create("whitelist.txt").unwrap();
        let _ = serde_json::to_writer_pretty(writer, &self.entries);
    }

}

impl WhitelistEntryProvider for JsonEntryProvider
{    
    fn new() -> Self
    {
        let reader = File::open("whitelist.txt");
        if let Ok(file) = reader
        {
            return JsonEntryProvider
                    {
                    entries : serde_json::from_reader(file).unwrap_or_else(|_| Vec::new())
                    }

        }
        else
        {
            return JsonEntryProvider
            {
                entries: Vec::new()
            }
        }
    }
    
    fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<WhitelistEntry> 
    { 
        for e in self.entries.iter()
        {
            if e.identification_token_id.cmp(&identity_token_id) == Ordering::Equal
            {
                return Some(e.clone());
            }
        }
        return None;
    }

    fn put_entry(&mut self, entry: WhitelistEntry) 
    { 
        self.entries.retain(|x| x.identification_token_id.cmp(&entry.identification_token_id) != Ordering::Equal);
        self.entries.push(entry);
        self.update_storage();
    }

    fn delete_entry(&mut self, identity_token_id: Vec<u8>) 
    { 
        self.entries.retain(|x| x.identification_token_id.cmp(&identity_token_id) != Ordering::Equal);
        self.update_storage();
    }
}


