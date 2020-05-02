
use serde::{Deserialize};
use std::{cmp::Ordering, io::Read};

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

#[derive(Clone, Deserialize, Debug)]
pub struct WhitelistEntry
{
    pub access_token_id: Vec<u8>,
    //pub access_profiles: Vec<AccessProfile> // Note: This should be a ref to another table or similar
}

pub trait WhitelistEntryProvider
{
    fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<WhitelistEntry>;
    fn put_entry(&mut self,entry: WhitelistEntry);
    // fn delete_entry(&mut self, identity_token_id Vec<u8>);
    fn new() -> Self;
}


pub struct SqliteEntryProvider;

impl WhitelistEntryProvider for SqliteEntryProvider
{
    fn new() -> Self{
        SqliteEntryProvider
    }

    fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<WhitelistEntry>
    {
        None
    }

    fn put_entry(&mut self, entry: WhitelistEntry)
    {

    }    
    
}

pub struct JsonEntryProvider
{
    entries: Vec<WhitelistEntry>
}

impl JsonEntryProvider
{
    // fn new<T>(dataSource: T) -> Self
    //     where T: Read
    // {
    //     let entries_read = serde_json::from_reader(dataSource);

    //     let mut data: Vec<WhitelistEntry> = Vec::new();
    //     if entries_read.is_ok()
    //     {
    //         data = entries_read.unwrap();
    //     }


    //     JsonEntryProvider
    //     {
    //         entries: data
    //     }
    // }

}

impl WhitelistEntryProvider for JsonEntryProvider
{    
    fn new() -> Self
    {
        JsonEntryProvider
        {
            entries: Vec::new()
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
        // ToDo:
        // - Check if entry is already available, if so update

        self.entries.push(entry);
    }
}


