use barracuda_core::core;
use barracuda_core::launch;
use barracuda_core::launch_impl;

mod io;

fn main() {
    // Note: Launch never returns!
    launch!(barracuda_core::trace::launch,
            barracuda_core::cfg::rest::launch,
            barracuda_core::acm::generic_whitelist::launch::<barracuda_core::acm::generic_whitelist::whitelist::JsonEntryProvider>,
            barracuda_core::arm::console_input::launch,
            barracuda_core::io::launch,
            barracuda_core::dcm::trivial::launch,
            crate::io::launch
            );    
}