use serde::{Deserialize, Serialize};

use super::{output_components::outputcomponentbase::OutputComponentSetting, input_components::{doorhandle::DoorHandle, dooropenerkey::DoorOpenerKey, framecontact::FrameContact, releasecontact::ReleaseContact}};

#[derive(Serialize,Deserialize, Clone)]
pub enum InputComponentSerialization
{
    FrameContact(FrameContact),
    DoorOpenerKey(DoorOpenerKey),
    DoorHandle(DoorHandle),
    ReleaseContact(ReleaseContact)
}

#[derive(Serialize,Deserialize, Clone)]
pub enum OutputComponentSerialization
{
    AlarmRelay(u32),
    ElectricStrike(OutputComponentSetting),
    AccessGranted(OutputComponentSetting)
}

#[derive(Serialize,Deserialize, Clone)]
pub struct PassagewaySetting
{
    pub id: u32,
    pub outputs: Vec<OutputComponentSerialization>,
    pub inputs: Vec<InputComponentSerialization>,
    pub access_points: Vec<u32>,
    // Denotes the time the door may be open until an alarm is triggered
    pub alarm_time: u64
}
