use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub struct TimeSlot
{
    pub day: Weekday,
    pub from: u32,
    pub to: u32
}

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