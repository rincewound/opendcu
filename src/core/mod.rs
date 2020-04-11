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
pub mod AtomicQueue;

#[derive(Clone, PartialEq)]
pub enum BootStage
{
    LowLevelInit,
    HighLevelInit,
    Application
}

#[derive(Clone)]
pub enum SystemMessage
{
    Shutdown,
    StageComplete(BootStage),
    RunStage(BootStage)
}