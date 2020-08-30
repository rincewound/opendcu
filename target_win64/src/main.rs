
extern crate barracuda_core;
extern crate generic_whitelist;

use barracuda_core::core;
use barracuda_core::launch;
use barracuda_core::launch_impl;

mod io;


fn main() {
    // Note: Launch never returns!
    launch!(barracuda_core::trace::launch,
            barracuda_core::cfg::rest::launch,
            generic_whitelist::launch::<generic_whitelist::whitelist::JsonEntryProvider>,
            barracuda_core::arm::console_input::launch,
            barracuda_core::io::launch,
            barracuda_core::dcm::trivial::launch,
            barracuda_core::profile::launch,
            crate::io::launch
            );    
}