
#[macro_use]
extern crate rouille;

//#[macro_use]
//extern crate diesel;

extern crate serde;

#[macro_use]
mod core;
#[macro_use]
mod cfg;

mod Trace;
mod acm;
mod arm;
mod sig;
mod dcm;
mod modcaps;


fn main() {
    // Note: Launch never returns!
    launch!(Trace::launch,
            crate::cfg::REST::launch,
            crate::acm::generic_whitelist::launch,
            crate::arm::console_input::launch)
            ;
}