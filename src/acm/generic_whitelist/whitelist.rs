
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, io::Read, fs::File};

// pub enum AccessFlags
// {
//     Access,
//     Pin
// }

// #[derive(Clone)]
// pub struct TimePair
// {
//     from: u16,
//     to: u16,
//     access_flags: u16
// }

// #[derive(Clone)]
// pub struct TimeProfile
// {
//     dayflags: u16,
//     time_pairs: Vec<TimePair>
// }

// #[derive(Clone)]
// pub struct AccessProfile
// {
//     // All access points, for which this profile is valid
//     access_points: Vec<u32>,
//     time_pro: TimeProfile
// }

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct WhitelistEntry
{
    pub access_token_id: Vec<u8>,
    //pub access_profiles: Vec<AccessProfile> // Note: This should be a ref to another table or similar
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
///  for all but the simplest and most statig production
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
        serde_json::to_writer_pretty(writer, &self.entries);
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
            if e.access_token_id.cmp(&identity_token_id) == Ordering::Equal
            {
                return Some(e.clone());
            }
        }
        return None;
    }

    fn put_entry(&mut self, entry: WhitelistEntry) 
    { 
        self.entries.retain(|x| x.access_token_id.cmp(&entry.access_token_id) != Ordering::Equal);
        self.entries.push(entry);
        self.update_storage();
    }

    fn delete_entry(&mut self, identity_token_id: Vec<u8>) 
    { 
        self.entries.retain(|x| x.access_token_id.cmp(&identity_token_id) != Ordering::Equal);
        self.update_storage();
    }
}


