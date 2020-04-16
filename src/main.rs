#[macro_use]
//extern crate diesel;

#[macro_use]
mod core;
mod Trace;

use crate::acm::WhitelistAccessRequest;
use self::core::ChannelManager::ChannelManager;

// Modules
mod acm;

fn main() {
/*
    Also, we'd like to automatically wire up the required channels without
    having to bother here. Maybe we should just generate the channels
    ad-hoc, if they are requested. 
    -> Not a good idea as it would require passing around mutable copies of
       the channel manager.
*/

    // This instance of the chm cannot be used!
    //let mut chm = ChannelManager::new();
    //chm.register_channel::<Trace::trace_message>();
    // chm.register_channel::<core::SystemMessage>();
    // chm.register_channel::<WhitelistAccessRequest>();

    // Trace::launch(&mut chm);

    // let tracer = Trace::TraceHelper::TraceHelper::new(String::from("Main"), &chm);
    // tracer.TraceStr("..welcome to barracuda. Starting up.");

    // Note: Launch never returns!
    launch!(Trace::launch
    /*crate::acm::generic_whitelist::launch*/);
}