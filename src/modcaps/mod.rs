
#[derive(Copy, Clone, Debug)]
pub enum ModuleCapability
{
    _Inputs(u32),            // Number of digital ins provided by this module
    _Outputs(u32),           // Number of digital outs provided by this module
    _AccessPoints(u32),      // Number of access points provided by this module
    _KeypadEntry(u32),       // Number of keypad instances provided by this module
    _Whitelist,
    // VirtualNetwork,      // At some point!
}


pub struct ModuleCapabilityAdvertisement
{
    module_id: u32,
    caps: Vec<ModuleCapability>
}