#[macro_use]
extern crate rouille;

extern crate serde;

extern crate barracuda_hal;


#[macro_use]
pub mod cfg;

pub mod acm;
pub mod arm;
pub mod sig;
pub mod dcm;
pub mod modcaps;
pub mod io;
pub mod profile;
pub mod events;
pub mod modulebase;