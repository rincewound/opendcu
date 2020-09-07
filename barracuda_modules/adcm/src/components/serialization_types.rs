use crate::components::framecontact::FrameContact;
use crate::components::outputcomponentbase::OutputComponentSetting;
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
pub enum InputComponentSerialization
{
    FrameContact(FrameContact)
}

#[derive(Serialize,Deserialize)]
pub enum OutputComponentSerialization
{
    ElectricStrike(OutputComponentSetting),
    AccessGranted(OutputComponentSetting)
}

#[derive(Serialize,Deserialize)]
pub struct PassagewaySetting
{
    pub id: u32,
    pub outputs: Vec<OutputComponentSerialization>,
    pub inputs: Vec<InputComponentSerialization>
}
