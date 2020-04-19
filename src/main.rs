#[macro_use]
//extern crate diesel;

#[macro_use]
mod core;
mod Trace;
mod acm;
mod arm;
mod sig;
mod dcm;

fn main() {
    // Note: Launch never returns!
    launch!(Trace::launch,
            crate::acm::generic_whitelist::launch,
            crate::arm::console_input::launch);
}