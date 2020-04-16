/*

The barracuda core services module contains the infrastructure
for the rest of the appliaction, most notably

* The implementation of the channel manager

*/

pub mod BroadcastChannel;
pub mod ChannelManager;
pub mod Event;
pub mod AtomicQueue;
pub mod Supervisor;

#[derive(Clone, Copy, PartialEq)]
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
    StageComplete(BootStage, u32),
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


macro_rules! wait_for {
    ($evt: ident, $id: expr, $head: expr) => (
        {
             if $head.has_data() { 
                 ($id)
             }
             else
             {
                $head.set_data_trigger($evt.clone(), $id);
                ($evt.wait())
            }
        }
    );
    ($evt: ident, $id: expr, $head: expr, $($tail: expr),+) =>(
        {
             if $head.has_data()
             {
                 ($id)
             }
             else
             {
                $head.set_data_trigger($evt.clone(), $id);
                (wait_for!($evt,$id+1, $($tail),+))
            }
        }
    )
}

macro_rules! select_chan {
    ($($channels: expr),+) => (
        {
            let mut evt = Arc::new(DataEvent::<u32>::new());
            (wait_for!(evt, 0, $($channels),+))
        }
    );
}


macro_rules! wait_for_with_timeout {
    ($evt: expr, $timeout: expr, $id: expr, $head: expr) => (
        {
            if $head.has_data() { 
                Some($id)
            }
            else
            {
                $head.set_data_trigger($evt, $id);
                ($evt.wait_with_timeout($timeout))
            }
        }
    );
    ($evt: expr, $timeout: expr, $id: expr, $head: expr, $($tail: expr),+) =>(
        {
            if $head.has_data()
            {
                Some($id)
            }
            else
            {
                $head.set_data_trigger($evt, $id);
                (wait_for_with_timeout!($evt, $timeout, $id + 1, $($tail),+))
            }
        }
    )
}

macro_rules! select_chan_with_timeout {
    ($timeout: expr, $($channels: expr),+) => (wait_for_with_timeout!(Arc::new(DataEvent::<u32>::new()), $timeout, 0, $($channels),+));
}