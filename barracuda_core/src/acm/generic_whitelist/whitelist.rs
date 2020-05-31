
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering};

use crate::util::{JsonStorage, ObjectStorage};


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

