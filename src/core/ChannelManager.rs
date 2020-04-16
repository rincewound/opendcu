
use crate::core::BroadcastChannel::*;
extern crate anymap;
use anymap::AnyMap;
use std::sync::Arc;

struct ChannelContainer<T: Clone> {
    tx: GenericSender<T>,
    rx: Arc<GenericReceiver<T>>
}

pub struct ChannelManager {
    channels: anymap::Map,
}

impl ChannelManager  {

    pub fn new() -> Self {
        let res = ChannelManager {
            channels: AnyMap::new(),
        };
        res
    }

    fn get_channel<T: 'static + Clone>(&mut self) -> &ChannelContainer<T>
    {
        if !self.channels.contains::<ChannelContainer<T>>()
        {
            let (_tx, _rx) = make_chan();
            let cont = ChannelContainer::<T> { tx: _tx, rx: _rx };
            self.channels.insert(cont);
        }

        self.channels.get::<ChannelContainer<T>>().unwrap()
    }

    pub fn get_receiver<T: 'static + Clone>(&mut self) -> Arc<GenericReceiver<T>> 
    {
        let container = self.get_channel::<T>();
        return container.rx.clone_receiver();
    }

    pub fn get_sender<T: 'static + Clone>(&mut self) -> GenericSender<T> 
    {
        let container = self.get_channel::<T>();
        return container.tx.clone();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ChannelManager::*;

    #[test]
    fn broadcast_works_as_intended()
    {
        let mut ch = ChannelManager::new();
        let tx = ch.get_sender::<i32>();
        let rx1 = ch.get_receiver::<i32>();
        let rx2 = ch.get_receiver::<i32>();
        let rx3 = rx1.clone_receiver();

        tx.send(4711);
        assert_eq!(4711, rx1.receive());
        assert_eq!(4711, rx3.receive());
        assert_eq!(4711, rx2.receive());
    }


    #[test]
    fn can_use_multiple_senders()
    {
        let mut ch = ChannelManager::new();
        let tx1 = ch.get_sender::<i32>();
        let tx2 = ch.get_sender::<i32>();
        let rx1 = ch.get_receiver::<i32>();

        tx1.send(4711);
        tx2.send(4951);
        assert_eq!(4711, rx1.receive());
        assert_eq!(4951, rx1.receive())       
    }
}
