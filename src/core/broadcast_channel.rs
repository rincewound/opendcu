use std::sync::{Arc, Weak};
use std::cell::*;
use super::{event::DataEvent, atomic_queue::AtomicQueue, shareable::Shareable};



const Garbage_Threshold: u32 = 10;

/*
ToDo:

* Channels should occiasionally clean the rec_queue of any 
* died references.
*/

pub struct ChannelImpl<T: Clone>
{
    receiver_queues:  Cell<Vec<Weak<GenericReceiver<T>>>>
}

impl <T: Clone> ChannelImpl<T>
{
    pub fn new() -> Self{
        ChannelImpl
        {        
            receiver_queues: Cell::new(Vec::new())
        }
    }

    pub fn push_message(&self, data: T)
    {
        let mut the_vec = self.receiver_queues.take();
        let mut garbage = 0;
        for i in the_vec.iter()
        {
           if let Some(owned) = i.upgrade()
           {
                owned.push_message(data.clone());
           }
           else
           {
               garbage += 1;
           }
        }

        // The garbage variable contains the number of "dead"
        // references in the receiver queue, i.e. weak-refs that
        // no longer point to alive objects. Since these still
        // use memory and slow down processing of this function
        // we will collect the garbage, whenever it passes a 
        // threshold
        if garbage > Garbage_Threshold
        {
            the_vec.retain(|x| x.upgrade().is_some());
        }

        self.receiver_queues.set(the_vec);
    }
}

pub fn make_receiver<T: Clone>(owner: Shareable<RefCell<ChannelImpl<T>>>) -> Arc<GenericReceiver<T>>
{
    let rec = Arc::new(GenericReceiver::<T>::new(owner.clone()));
    let weak = Arc::downgrade(&rec.clone());
    let mut rec_queue = owner.lock().unwrap();
    let m = rec_queue.get_mut();
    m.receiver_queues.get_mut().push(weak);
    rec
}

pub fn make_sender<T: Clone>(owner: Shareable<RefCell<ChannelImpl<T>>>) -> GenericSender<T>
{
    GenericSender::<T>::new(owner)
}

pub fn make_chan<T: Clone>() -> (GenericSender<T>, Arc<GenericReceiver<T>>)
{
    let chan = Shareable::new(RefCell::new(ChannelImpl::<T>::new()));
    let receiver = make_receiver(chan.clone());
    let sender = make_sender(chan);        
    (sender, receiver)
}

pub struct GenericReceiver<T: Clone>
{
    owner: Shareable<RefCell<ChannelImpl<T>>>,
    data: AtomicQueue<T>
}

impl <T: Clone> GenericReceiver<T>
{
    pub fn new(owner: Shareable<RefCell<ChannelImpl<T>>>) -> Self
    {
        GenericReceiver
        {
            owner: owner,
            data: AtomicQueue::<T>::new()
        }
    }

    pub fn create_sender(&self) -> GenericSender<T>
    {
        return make_sender(self.owner.clone());
    }

    pub fn clone_receiver(&self) -> Arc<Self>
    {
        make_receiver(self.owner.clone())
    }

    pub fn push_message(&self, data: T)
    {
        self.data.push(data);
    }

    pub fn has_data(&self) -> bool
    {
        return self.data.len() != 0;
    }

    pub fn receive(&self) -> T
    {
        let mut result: Option<T> = None;
        /*
            Note: Depending on how data arrives,
            there are situations where we actually
            receive a None from the Queue!
        */
        while result.is_none()
        {
            if self.data.len() == 0
            {
                self.data.wait_data();
            }
            result = self.data.pop();  
        }
        return result.unwrap();   
    }

    pub fn receive_with_timeout(&self, milliseconds: u64) -> Option<T>
    {
        if self.data.len() != 0
        {
            return self.data.pop();
        }

        self.data.wait_with_timeout(milliseconds);
        return self.data.pop();
    }

    pub fn set_data_trigger(&self, d: Arc<DataEvent<u32>>, trigger_data:u32)
    {
        self.data.set_data_trigger(d, trigger_data)
    }
}



pub struct GenericSender<T: Clone>
{
    source: Shareable<RefCell<ChannelImpl<T> >>
}

impl <T: Clone> GenericSender<T>
{
    pub fn new(owner: Shareable<RefCell<ChannelImpl<T> >>) -> Self{
        GenericSender {
            source: owner.clone()
        }
    }

    pub fn send(&self, data: T)
    {
        self.source.lock().unwrap().get_mut().push_message(data.clone());
    }

    pub fn clone(&self) -> Self
    {
        GenericSender{
            source: self.source.clone()
        }
    }
}


#[cfg(test)]
mod tests {
     use crate::core::broadcast_channel::*;


    #[test]
    fn can_create_channel()
    {
        let (tx, rx) = make_chan();
        tx.send(24);
        assert_eq!(24, rx.receive())
    }

    #[test]
    fn cloned_receiver_receives_all_messages()
    {
        let (tx, rx) = make_chan();        
        let rx2 = rx.clone_receiver();
        tx.send(24);
        assert_eq!(24, rx.receive());
        assert_eq!(24, rx2.receive())
    }

    #[test]
    fn can_use_multiple_senders()
    {
        let (tx, rx) = make_chan();
        let tx2 = tx.clone();       
        tx.send(24);
        tx2.send(42);
        assert_eq!(24, rx.receive());
        assert_eq!(42, rx.receive())
    }

    #[test]
    fn receive_with_timeout_yields_none_after_timeout()
    {
        let (tx, rx) = make_chan();
        tx.send(24);
        rx.receive();
        assert!(None == rx.receive_with_timeout(50))
    }

    #[test]
    fn receive_with_timeout_yields_some_after_timeout()
    {
        let (tx, rx) = make_chan();
        tx.send(24);
        assert!(Some(24) == rx.receive_with_timeout(50))
    }

    // #[test]
    // fn can_use_select_macro()
    // {
    //     let (tx, rx) = make_chan();
    //     tx.send(1);
    //     let chanid = select_chan!(rx);
    //     assert_eq!(0, chanid);
    // }

    // #[test]
    // fn can_use_select_macro_for_multiple_channels()
    // {
    //     let (tx1, rx1) = make_chan();
    //     let (tx2, rx2) = make_chan();
    //     let (tx3, rx3) = make_chan();
    //     tx2.send(11.2);
    //     let chanid = select_chan!(rx1, rx2, rx3);
    //     assert_eq!(1, chanid);
    //     tx1.send(10);
    //     tx3.send("foo".to_string());
    // }
}
