/*

The barracuda core services module contains the infrastructure
for the rest of the appliaction, most notably

* The implementation of the channel manager
* The implementation of the checkpoint system
* Logging/Tracing

*/

pub mod BroadcastChannel;
pub mod ChannelManager;
pub mod Event;
pub mod AtomicQueue;
pub mod Supervisor;

#[derive(Clone, PartialEq)]
pub enum BootStage
{
    Sync,
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

macro_rules! launch {
    ($($threadlist: expr),+) => (
        {
            let mut supervisor = crate::core::Supervisor::Supervisor::new();
            launch_impl!(supervisor, $($threadlist),+);            
            supervisor.run();
        }
    )
}


macro_rules! launch_impl {
    ($supervisor: expr, $head: expr, $($threadlist: expr),+) => (
        {
            //core::Supervisor::start_thread($head);
            $supervisor.start_thread($head);
            launch_impl!($supervisor, $($threadlist),+)                    
        }
    );
    ($supervisor: expr, $head: expr) => (
        {
            $supervisor.start_thread($head);         
        }
    )
}