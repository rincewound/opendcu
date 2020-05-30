

use super::whitelist::WhitelistEntry;
use serde::{Deserialize, Serialize};
use crate::util::{JsonStorage, ObjectStorage};
use chrono::{DateTime, Datelike, Timelike, Local};
use strum_macros::*;

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
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

#[derive(Clone, Copy, Debug, Display)]
pub enum ProfileCheckResult
{
    NoAccessRights,     // Emitted, if the profile does not contain the AP requested
    TimezoneViolated,   // Emitted, if the profile contains the AP but booking is outside of time_pro
    InvalidProfile      // Emitted, if Whitelist entry refers to unknown profile
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub struct TimeSlot
{
    pub day: Weekday,
    pub from: u32,
    pub to: u32
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
    fn check_profile(&self, ap_id: u32, entry: &WhitelistEntry) -> Result<(), ProfileCheckResult>;
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
    fn check_profile(&self, ap_id: u32, entry: &WhitelistEntry) -> Result<(), ProfileCheckResult> 
    {
        for profile_id in entry.access_profiles.iter()
        {
            let profile = self.profiles.get_entry(|x| x.id == *profile_id);
            if let Some(the_profile) = profile
            {   
                let datetime = Local::now();
                return check_profile_impl(ap_id, entry, &the_profile, datetime)              
            }
        }
        return Err(ProfileCheckResult::InvalidProfile);
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


fn check_profile_impl<T>(ap_id: u32, entry: &WhitelistEntry, profile: &AccessProfile, now: DateTime<T>) -> Result<(),ProfileCheckResult> 
    where T: chrono::TimeZone
{
    // ToDo: All Profiles assume "local time", whatever that means. We should be timezone aware.
    if !profile.access_points.contains(&ap_id) { return Err(ProfileCheckResult::NoAccessRights); }

    let weekday = now.weekday() as i32;
    for slot in profile.time_pro.iter()
    {
        // Check if day matches for timeslot
        if slot.day as u32 != weekday as u32 { 
            continue; 
        }

        // check if time matches:
        let industry_from = now.hour() * 100 + now.minute();
        if industry_from >= slot.from && industry_from <= slot.to
        {
            return Ok(());
        }
    }

    return Err(ProfileCheckResult::TimezoneViolated);
}

#[cfg(test)]
mod tests {

    use crate::acm::generic_whitelist::whitelist::WhitelistEntry;
    use super::{TimeSlot, AccessProfile, check_profile_impl};
    use chrono::{DateTime};

    #[test]
    fn check_profile_yields_true_if_valid_profile()
    {
        let entry = WhitelistEntry{identification_token_id: vec![], access_profiles: vec![1]};
        let profile = AccessProfile {id: 0, access_points: vec![1,2], time_pro: vec![
            TimeSlot{day: super::Weekday::Monday, from: 700, to: 1000}
        ]};

        let dt = DateTime::parse_from_rfc3339("2020-05-25T08:00:57-08:00").unwrap();
        assert!(check_profile_impl(1, &entry, &profile, dt).is_ok());
    }

    #[test]
    fn check_profile_yields_false_if_bad_day()
    {
        let entry = WhitelistEntry{identification_token_id: vec![], access_profiles: vec![1]};
        let profile = AccessProfile {id: 0, access_points: vec![1,2], time_pro: vec![
            TimeSlot{day: super::Weekday::Monday, from: 700, to: 1000}
        ]};
        // Tue, 26.5.2020, 8:00 AM
        let dt = DateTime::parse_from_rfc3339("2020-05-26T08:00:57-08:00").unwrap();
        assert!(check_profile_impl(1, &entry, &profile, dt).is_err());
    }

    #[test]
    fn check_profile_yields_false_if_bad_time_slot()
    {
        let entry = WhitelistEntry{identification_token_id: vec![], access_profiles: vec![1]};
        let profile = AccessProfile {id: 0, access_points: vec![1,2], time_pro: vec![
            TimeSlot{day: super::Weekday::Monday, from: 700, to: 1000}
        ]};
        // Mon, 25.5.2020, 11:00 AM
        let dt = DateTime::parse_from_rfc3339("2020-05-25T11:00:57-08:00").unwrap();
        assert!(check_profile_impl(1, &entry, &profile, dt).is_err());
    }

    #[test]
    fn check_profile_yields_false_if_bad_access_point()
    {
        let entry = WhitelistEntry{identification_token_id: vec![], access_profiles: vec![1]};
        let profile = AccessProfile {id: 0, access_points: vec![1,2], time_pro: vec![
            TimeSlot{day: super::Weekday::Monday, from: 700, to: 1000}
        ]};
        // Mon, 25.5.2020, 8:00 AM
        let dt = DateTime::parse_from_rfc3339("2020-05-25T08:00:57-08:00").unwrap();
        assert!(true, check_profile_impl(5, &entry, &profile, dt).is_err());
    }
}