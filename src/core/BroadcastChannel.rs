use std::sync::{Arc, Mutex};
use std::cell::*;
use std::collections::VecDeque;
use crate::core::Event::*;

pub struct ChannelImpl<T: Clone>
{
    receiver_queues:  Cell<Vec<Arc<Mutex<Cell<VecDeque<T>>>>>>
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
        let theVec = self.receiver_queues.take();
        for i in theVec.iter()
        {
            i.lock().unwrap().get_mut().push_back(data.clone())
        }
        self.receiver_queues.set(theVec);
    }

    pub fn make_receiver(owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>) -> GenericReceiver<T>
    {
        let q = Arc::new(Mutex::new(Cell::new(VecDeque::new())));
        let rc = GenericReceiver::<T>::new(owner.clone(), q.clone());

        let mut recQueue = owner.lock().unwrap();
        let m = recQueue.get_mut();
        m.receiver_queues.get_mut().push(q.clone());
        rc
    }

    pub fn make_sender(owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>) -> GenericSender<T>
    {
        GenericSender::<T>::new(owner)
    }

    pub fn make_chan() -> (GenericSender<T>, GenericReceiver<T>)
    {
        let chan = Arc::new(Mutex::new(RefCell::new(ChannelImpl::<T>::new())));
        let receiver = ChannelImpl::make_receiver(chan.clone());
        let sender = ChannelImpl::make_sender(chan);        
        (sender, receiver)
    }
}

pub struct GenericReceiver<T: Clone>
{
    data_event: Event,
    owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>,
    data: Arc<Mutex<Cell<VecDeque<T>>>>
}

impl <T: Clone> GenericReceiver<T>
{
    pub fn new(owner: Arc<Mutex<RefCell<ChannelImpl<T>>>>, _data: Arc<Mutex<Cell<VecDeque<T>>>> ) -> Self
    {
        GenericReceiver
        {
            data_event: Event::new(),
            owner: owner,
            data: _data
        }
    }

    pub fn clone(&self) -> Self
    {
        ChannelImpl::make_receiver(self.owner.clone())
    }

    pub fn push_message(&self, data: T)
    {
        let mut d = self.data.lock().unwrap();
        d.get_mut().push_back(data.clone());
        self.data_event.trigger();
    }

    pub fn receive(&self) -> T
    {
        {
            let mut data = self.data.lock()
                                    .unwrap();
            let mutable_d = data.get_mut();
            if mutable_d.len() != 0
            {
                return mutable_d.pop_front().unwrap()
            }
        }
        
        // if we got here, data is currently empty. We wait.
        self.data_event.wait();
        
        let mut data = self.data.lock()
                                .unwrap();
        let mutable_d = data.get_mut();
        return mutable_d.pop_front().unwrap()     
    }

    pub fn receive_with_timeout(&self, milliseconds: u64) -> Option<T>
    {
        {
            let mut data = self.data.lock()
                                    .unwrap();
            let mutable_d = data.get_mut();
            if mutable_d.len() != 0
            {
                return Some(mutable_d.pop_front().unwrap())
            }
        }
        
        // if we got here, data is currently empty. We wait.
        if !self.data_event.wait_with_timeout(milliseconds)
        {
            return None
        }
        
        let mut data = self.data.lock()
                                .unwrap();
        let mutable_d = data.get_mut();
        return Some(mutable_d.pop_front().unwrap())
    }
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
        let (tx, rx) = ChannelImpl::make_chan();
        tx.send(24);
        assert_eq!(24, rx.receive())
    }

    #[test]
    fn cloned_receiver_receives_all_messages()
    {
        let (tx, rx) = ChannelImpl::make_chan();        
        let rx2 = rx.clone();
        tx.send(24);
        assert_eq!(24, rx.receive());
        assert_eq!(24, rx2.receive())
    }

    #[test]
    fn receive_with_timeout_yields_none_after_timeout()
    {
        let (tx, rx) = ChannelImpl::make_chan();
        tx.send(24);
        rx.receive();
        assert!(None == rx.receive_with_timeout(50))
    }

    #[test]
    fn receive_with_timeout_yields_some_after_timeout()
    {
        let (tx, rx) = ChannelImpl::make_chan();
        tx.send(24);
        assert!(Some(24) == rx.receive_with_timeout(50))
    }
}
