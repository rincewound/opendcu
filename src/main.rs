#![feature(fn_traits)]

#[macro_use]
extern crate rouille;

//#[macro_use]
//extern crate diesel;

extern crate serde;

#[macro_use]
mod core;
#[macro_use]
mod cfg;

mod trace;
mod acm;
mod arm;
mod sig;
mod dcm;
mod modcaps;
mod io;

// Platform specific
mod platform;



fn main() {
    // Note: Launch never returns!
    launch!(trace::launch,
            crate::cfg::REST::launch,
            crate::acm::generic_whitelist::launch::<acm::generic_whitelist::whitelist::JsonEntryProvider>,
            crate::arm::console_input::launch,
            crate::io::launch,
            crate::dcm::trivial::launch,
            crate::platform::win64::launch
            );
}