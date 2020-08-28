use crate::util::{datetime::TimeSlot, JsonStorage, ObjectStorage};
use serde::{Deserialize, Serialize};
use chrono::{Datelike, Timelike};
use super::ProfileChangeEvent;

#[derive(Clone, Serialize, Deserialize)]
struct BinaryProfile
{
    id: u32,
    time_slots: Vec<TimeSlot>
}

pub struct ProfileChecker
{
    profiles: JsonStorage<BinaryProfile>
}

impl ProfileChecker
{
    pub fn new() -> Self
    {
        Self
        {
            profiles: JsonStorage::new("./profiles.txt".to_string())
        }
    }

    fn is_profile_active(prof: &BinaryProfile, time: chrono::naive::NaiveDateTime) -> bool
    {
        for slot in prof.time_slots.iter().filter(|x| x.day as u32 == time.weekday() as u32)
        {
            let industry_time = time.hour() * 100 + time.minute();
            return industry_time >= slot.from && industry_time <= slot.to;
        }
        return false;
    }

    pub fn tick(&mut self, now: chrono::naive::NaiveDateTime, last_time: chrono::naive::NaiveDateTime) -> Vec<ProfileChangeEvent>
    {
        let mut result = Vec::new();
        for profile in self.profiles.iter()
        {
            // check if profile has a different state now than it had last_time, if
            // so trigger event.
            let is_active_now = ProfileChecker::is_profile_active(profile, now);
            let was_profile_active = ProfileChecker::is_profile_active(profile, last_time);

            if is_active_now != was_profile_active
            {
                let evt = ProfileChangeEvent {
                    profile_id: profile.id,
                    profile_state: if is_active_now { super::ProfileState::Active } else { super::ProfileState::Inactive}
                };
                result.push(evt);
            }

        }

        result
    }
}