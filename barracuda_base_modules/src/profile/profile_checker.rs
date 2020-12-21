
use barracuda_core::util::{JsonStorage, datetime::TimeSlot, ObjectStorage};
use serde::{Deserialize, Serialize};
use chrono::{Datelike, Timelike, Local};
use super::ProfileChangeEvent;

#[derive(Clone, Serialize, Deserialize)]
pub struct BinaryProfile
{
    pub id: u32,
    pub time_slots: Vec<TimeSlot>
}

pub struct ProfileChecker
{
    profiles: JsonStorage<BinaryProfile>,
    first_check_done: bool
}

impl ProfileChecker
{
    pub fn new() -> Self
    {
        Self
        {
            profiles: JsonStorage::new("./bin_profiles.txt".to_string()),
            first_check_done: false
        }
    }

    fn is_profile_active(prof: &BinaryProfile, time: chrono::DateTime<Local>) -> bool
    {
        for slot in prof.time_slots.iter().filter(|x| x.day as u32 == time.weekday() as u32)
        {
            let industry_time = time.hour() * 100 + time.minute();
            return industry_time >= slot.from && industry_time <= slot.to;
        }
        return false;
    }

    pub fn tick(&mut self, now: chrono::DateTime<Local>, last_time: chrono::DateTime<Local>) -> Vec<ProfileChangeEvent>
    {
        let mut result = Vec::new();
        for profile in self.profiles.iter()
        {
            // check if profile has a different state now than it had last_time, if
            // so trigger event.
            let is_active_now = ProfileChecker::is_profile_active(profile, now);
            let was_profile_active = ProfileChecker::is_profile_active(profile, last_time);

            // The "first_check_done" flag is used to always generate an event for the profiles on
            // startup
            if (is_active_now != was_profile_active) || !self.first_check_done
            {
                let evt = ProfileChangeEvent {
                    profile_id: profile.id,
                    profile_state: if is_active_now { super::ProfileState::Active } else { super::ProfileState::Inactive}
                };
                result.push(evt);
            }
        }

        self.first_check_done = true;
        result
    }

    pub fn add_profile(&mut self, prof: BinaryProfile)
    {
        self.profiles.put_entry(prof);
        self.profiles.update_storage();
        // reset flag in order to see events for
        // the new profile on next tick!
        self.first_check_done = false;
    }

    pub fn delete_profile(&mut self, prof_id: u32)
    {
        self.profiles.delete_entry( |x| x.id == prof_id);
        self.profiles.update_storage();
    }
}