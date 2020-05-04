
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
mod webserver;

fn main() {
    // Note: Launch never returns!
    launch!(trace::launch,
            crate::cfg::REST::launch,
            crate::acm::generic_whitelist::launch::<acm::generic_whitelist::whitelist::JsonEntryProvider>,
            crate::arm::console_input::launch,
            crate::webserver::launch)
            ;
}