/*

The barracuda core services module contains the infrastructure
for the rest of the appliaction, most notably

* The implementation of the channel manager

*/

use std::sync::{Arc, Mutex};
use crate::modcaps::*;

pub mod BroadcastChannel;
pub mod ChannelManager;
pub mod Event;
pub mod AtomicQueue;
pub mod Supervisor;
pub mod BootstageHelper;

#[derive(Clone, Copy, PartialEq)]
pub enum BootStage
{
    Sync,
    Advertise,
    LowLevelInit,
    HighLevelInit,
    Application
}

#[derive(Clone)]
pub enum SystemMessage
{
    Shutdown,
    StageComplete(BootStage, u32),
    Advertisement(ModuleCapability),
    RunStage(BootStage),
    RegisterConfigInterface(Arc<Mutex<i32>>)        // Note that we might not actually need this, if CFG is just another module.
}

/**
Launch expects a list of functions. Launch will call all
functions and walk through the bootup sequence, expecting
all called functions to:
    start a thread that implements the bootup protocol, 
    i.e. it shall check in when a RunStage command is
    sent.

    Panics if any of the stages fail to check in within
    2.5 seconds after a run-stage command, except for
    the "Application" stage which is not expected to
    be answered.
**/
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
            let evt = Arc::new(DataEvent::<u32>::new());
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