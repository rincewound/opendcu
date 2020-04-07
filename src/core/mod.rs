/*

The barracuda core services module contains the infrastructure
for the rest of the appliaction, most notably

* The implementation of the channel manager
* The implementation of the checkpoint system
* Logging/Tracing

*/

pub mod BroadcastChannel;
pub mod ChannelManager;
pub mod Checkpoint;
pub mod Event;