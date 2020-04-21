
#[derive(Copy, Clone, Debug)]
pub enum ModuleCapability
{
    Inputs(u32),            // Number of digital ins provided by this module
    Outputs(u32),           // Number of digital outs provided by this module
    AccessPoints(u32),      // Number of access points provided by this module
    KeypadEntry(u32),       // Number of keypad instances provided by this module
    Whitelist,
    // VirtualNetwork,      // At some point!
}


pub struct ModuleCapabilityAdvertisement
{
    module_id: u32,
    caps: Vec<ModuleCapability>
}