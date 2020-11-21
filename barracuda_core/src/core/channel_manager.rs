
use crate::core::broadcast_channel::*;
extern crate anymap;
use anymap::AnyMap;
use std::sync::Arc;
use crate::core::shareable::Shareable;


struct ChannelContainer<T: Clone> {
    tx: GenericSender<T>,
    rx: Arc<GenericReceiver<T>>
}

pub struct ChannelManager {
    channels: Shareable<anymap::Map>,
}

impl ChannelManager  {

    pub fn new() -> Self {
        let res = ChannelManager {
            channels: Shareable::new(AnyMap::new()),
        };
        res
    }

    fn ensure_channel_exists<T: 'static + Clone>(&mut self)
    {
        let mut writeable_channels = self.channels.lock();
        if !writeable_channels.contains::<ChannelContainer<T>>()
        {
            let (_tx, _rx) = make_chan();
            let cont = ChannelContainer::<T> { tx: _tx, rx: _rx };
            writeable_channels.insert(cont);
        }
    }

    pub fn get_receiver<T: 'static + Clone>(&mut self) -> Arc<GenericReceiver<T>> 
    {
        self.ensure_channel_exists::<T>();
        return self.channels.lock().get::<ChannelContainer<T>>()
                                   .unwrap()
                                   .rx
                                   .clone();
    }

    pub fn get_sender<T: 'static + Clone>(&mut self) -> GenericSender<T> 
    {
        self.ensure_channel_exists::<T>();
        return self.channels.lock().get::<ChannelContainer<T>>()
                                   .unwrap()
                                   .tx
                                   .clone();
    }
}

impl Clone for ChannelManager
{
    fn clone(&self) -> Self {
        Self
        {
            channels: self.channels.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::channel_manager::*;

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
