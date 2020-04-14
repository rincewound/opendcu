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


/*
macro_rules! wait_for {
    ($evt: expr, $id: expr, $head: expr) => (
        {
            if $head.has_data() { 
                ($id)
            }
            else
            {
                $head.set_data_trigger($evt, $id);
                ($evt.wait())
            }
        }
    );
    ($evt: expr, $id: expr, $head: expr, $($tail: expr),+) =>(
        {
            if $head.has_data()
            {
                ($id)
            }
            else
            {
                $head.set_data_trigger($evt, $id);
                (wait_for!($evt,$id+1, $($tail),+))
            }
        }
    )
}
*/

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
            $supervisor.start_thread($head)
            launch_impl!($supervisor, $($threadlist),+)                    
        }
    );
    ($supervisor: expr, $head: expr) => (
        {
            $supervisor.start_thread($head);         
        }
    )
}