/*

The barracuda core services module contains the infrastructure
for the rest of the appliaction, most notably

* The implementation of the channel manager

*/

use std::sync::{Arc, Mutex};
use crate::modcaps::*;

pub mod broadcast_channel;
pub mod channel_manager;
pub mod event;
pub mod atomic_queue;
pub mod supervisor;
pub mod bootstage_helper;

#[derive(Clone, Copy, PartialEq)]
pub enum BootStage
{
    Sync,
    _Advertise,
    LowLevelInit,
    HighLevelInit,
    Application
}

#[derive(Clone)]
pub enum SystemMessage
{
    Shutdown,
    StageComplete(BootStage, u32),
    _Advertisement(ModuleCapability),
    RunStage(BootStage),
    _RegisterConfigInterface(Arc<Mutex<i32>>)        // Note that we might not actually need this, if CFG is just another module.
}


// # Generate a System Unique iD
// A module ID is a 16 Bit integer consisting of the acutal id and the instancenumber of the module:
// AAAA AAAA BBBB BBBB, where:
// A: Identifies the actual implementation of the module, this is a value that should be unique to each moduleimplementation (i.e. two different kinds of ARM shall have different IDs!)
// B: Identifies the instancenumber of the module (if a module is started multiple times, this number shall count up!).
// A component ID is a 32 bit int, that consists of the ID of the module owning the component and 16 bits containing,
// the index of the component within the module:
// AAAA AAAA BBBB BBBB CCCC CCCC CCCC CCCC
// C: Used to uniquely identify a given component of a module
pub fn make_sud(module_id: u8, module_instance: u8, object_index: u16) -> u32
{
    let m = (module_id as u32) << 24;
    let i = (module_instance as u32) << 16;
    return m | i | (object_index as u32);
}

pub fn modid_from_sud(sud: u32) -> u32
{
    return sud >> 16;
}

pub fn objectindex_from_sud(sud: u32) -> u32
{
    return sud & 0x0000FFFF;
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
            let mut supervisor = crate::core::supervisor::Supervisor::new();
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