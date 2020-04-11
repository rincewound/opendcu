/*
A checkpoint acts much like a barrier, except that
we do not know beforehand how many members will take
part in the CP. 

Usage: 

let cp = Checkpoint::new(CheckpointID::Id)
cp.arm()
cp.enter()

The CP system works as follows:
Each instance of a checkpoint with a given ID will send a "Registered" message
upon creation, notifying all other checkpoints with the same id

*/
//enum CheckpointProtocol
// {
//     Registered(i32),        // Sent by each checkpoint instance upon creation to notify other instances of the same kind
//     Entered(i32),
//     Freed(i32)
// }

// #[derive(Clone, Debug)]
// struct Checkpoint
// {
//     num_instances: i32
// }

// fn make_checkpoint() -> Checkpoint
// {
//     static initialized : bool = false;
// }

// impl Checkpoint 
// {
//     fn register(){}
//     fn arm(){}
//     fn enter(){}
// }