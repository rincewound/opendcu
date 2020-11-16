
extern crate barracuda_core;
extern crate barracuda_hal;
extern crate mfrc522;
extern crate generic_whitelist;
extern crate rppal;

use barracuda_core::core;
use barracuda_core::launch;
use barracuda_core::launch_impl;

mod drivers;
use drivers::*;

fn main() {
    // Note: Launch never returns!
    launch!(barracuda_core::trace::launch,
            barracuda_core::cfg::rest::launch,
            generic_whitelist::launch::<generic_whitelist::whitelist::JsonEntryProvider>,
            barracuda_core::arm::console_input::launch,
            barracuda_core::io::launch,
            adcm::launch,            
            |chm| mfrc522::launch(chm, RfidSpi::new(), RfidIrq::new()),
            barracuda_core::profile::launch
            );    
}