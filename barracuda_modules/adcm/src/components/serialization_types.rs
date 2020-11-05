use crate::components::framecontact::FrameContact;
use crate::components::dooropenerkey::DoorOpenerKey;
use crate::components::outputcomponentbase::OutputComponentSetting;
use crate::components::doorhandle::DoorHandle;
use crate::components::releasecontact::ReleaseContact;
use crate::components::alarmrelay::AlarmRelay;
use serde::{Deserialize, Serialize};

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
    pub access_points: Vec<u32>
}
