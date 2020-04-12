use std::sync::{Arc, Mutex, Weak};
use std::cell::*;
use super::{Event::DataEvent, AtomicQueue::AtomicQueue};

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
        let the_vec = self.receiver_queues.take();
        for i in the_vec.iter()
        {
           if let Some(owned) = i.upgrade()
           {
                owned.push_message(data.clone());
           }
        }
        self.receiver_queues.set(the_vec);
    }
}

pub fn make_receiver<T: Clone>(owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>) -> Arc<GenericReceiver<T>>
{
    let rec = Arc::new(GenericReceiver::<T>::new(owner.clone()));
    let weak = Arc::downgrade(&rec.clone());
    let mut rec_queue = owner.lock().unwrap();
    let m = rec_queue.get_mut();
    m.receiver_queues.get_mut().push(weak);
    rec
}

pub fn make_sender<T: Clone>(owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>) -> GenericSender<T>
{
    GenericSender::<T>::new(owner)
}

pub fn make_chan<T: Clone>() -> (GenericSender<T>, Arc<GenericReceiver<T>>)
{
    let chan = Arc::new(Mutex::new(RefCell::new(ChannelImpl::<T>::new())));
    let receiver = make_receiver(chan.clone());
    let sender = make_sender(chan);        
    (sender, receiver)
}

pub struct GenericReceiver<T: Clone>
{
    owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>,
    data: AtomicQueue<T>
}

impl <T: Clone> GenericReceiver<T>
{
    pub fn new(owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>) -> Self
    {
        GenericReceiver
        {
            owner: owner,
            data: AtomicQueue::<T>::new()
        }
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
        self.data.wait_data();
        return self.data.pop().unwrap();     
    }

    pub fn receive_with_timeout(&self, milliseconds: u64) -> Option<T>
    {
        if self.data.wait_with_timeout(milliseconds)
        {
            return self.data.pop();
        }
        return None
    }

    pub fn set_data_trigger(&self, d: Arc<DataEvent<u32>>, trigger_data:u32)
    {
        self.data.set_data_trigger(d, trigger_data)
    }
}



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

macro_rules! select_chan {
    ($($channels: expr),+) => (wait_for!(Arc::new(DataEvent::<u32>::new()), 0, $($channels),+));
}
pub struct GenericSender<T: Clone>
{
    source: Arc<Mutex<RefCell<ChannelImpl<T> >>>
}

impl <T: Clone> GenericSender<T>
{
    pub fn new(owner: Arc<Mutex<RefCell<ChannelImpl<T> >>>) -> Self{
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
     use crate::core::BroadcastChannel::*;

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

    #[test]
    fn can_use_select_macro()
    {
        let (tx, rx) = make_chan();
        tx.send(1);
        let chanid = select_chan!(rx);
        assert_eq!(0, chanid);
    }

    #[test]
    fn can_use_select_macro_for_multiple_channels()
    {
        let (tx1, rx1) = make_chan();
        let (tx2, rx2) = make_chan();
        let (tx3, rx3) = make_chan();
        tx2.send(11.2);
        let chanid = select_chan!(rx1, rx2, rx3);
        assert_eq!(1, chanid);
        tx1.send(10);
        tx3.send("foo".to_string());
    }
}
