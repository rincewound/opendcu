use barracuda_core::core;
use barracuda_core::launch;
use barracuda_core::launch_impl;

mod io;

fn main() {
    // Note: Launch never returns!
    launch!(barracuda_core::trace::launch,
            barracuda_base_modules::cfg::rest::launch,            
            barracuda_base_modules::arm::console_input::launch,
            barracuda_base_modules::io::launch,            
            barracuda_base_modules::profile::launch,
            generic_whitelist::launch::<generic_whitelist::whitelist::JsonEntryProvider>,
            adcm::launch,
            crate::io::launch
            );    
}