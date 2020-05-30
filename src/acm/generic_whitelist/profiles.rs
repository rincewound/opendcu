

use super::whitelist::WhitelistEntry;
use serde::{Deserialize, Serialize};
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

impl JsonProfileChecker
{
    pub fn new(file: String) -> Self{
        JsonProfileChecker{
            profiles: JsonStorage::new(file)
        }
    }
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
                if check_profile_impl(ap_id, entry, &the_profile)
                {
                    return true;
                }
                // if the_profile.access_points.iter().find(|&&x| x == ap_id).is_none() { break; }
                
                // // the ap is contained in the profile. Check the time_pro
                // // and be done with it.
                // for tp in the_profile.time_pro.iter()
                // {
                //     if tp.is_active()
                //     {
                //         return true;
                //     }
                // }
                
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

impl TimeSlot
{
    pub fn is_active(&self) -> bool
    {
        return true;
    }
}


fn check_profile_impl(ap_id: u32, entry: &WhitelistEntry, profile: &AccessProfile) -> bool {
    false
}

// #[cfg(test)]
// mod tests {

//     #[test]
//     fn check_profile_yields_true_if_valid_profile()
//     {
//         assert!(false)
//     }
// }