
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

    pub fn register_channel<T: 'static + Clone>(&mut self) {
        if self.channels.contains::<ChannelContainer<T>>()
        {
            return;
        }
        let (_tx, _rx) = make_chan();
        let cont = ChannelContainer::<T> { tx: _tx, rx: _rx };
        self.channels.insert(cont);
    }

    pub fn get_receiver<T: 'static + Clone>(&self) -> Option<Arc<GenericReceiver<T>>> {
        let container = self.channels.get::<ChannelContainer<T>>();
        if let Some(x) = container {
            return Some(x.rx.clone_receiver());
        }
        None
    }

    pub fn get_sender<T: 'static + Clone>(&self) -> Option<GenericSender<T>> {
        let container = self.channels.get::<ChannelContainer<T>>();
        if let Some(x) = container {
            return Some(x.tx.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ChannelManager::*;

    #[test]
    fn get_nonexisting_channel_yields_none() {
        let ch = ChannelManager::new();
        let sender = ch.get_sender::<i32>();
        match sender {
            None => return,
            _ => assert_eq!(true, false),
        }
    }

    #[test]
    fn can_register_channel() {
        let mut ch = ChannelManager::new();
        ch.register_channel::<i32>();
        let channel = ch.get_sender::<i32>();
        match channel {
            None => assert_eq!(true, false),
            _ => return,
        }
    }

    #[test]
    fn broadcast_works_as_intended()
    {
        let mut ch = ChannelManager::new();
        ch.register_channel::<i32>();
        let tx = ch.get_sender::<i32>().unwrap();
        let rx1 = ch.get_receiver::<i32>().unwrap();
        let rx2 = ch.get_receiver::<i32>().unwrap();
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
        ch.register_channel::<i32>();
        let tx1 = ch.get_sender::<i32>().unwrap();
        let tx2 = ch.get_sender::<i32>().unwrap();
        let rx1 = ch.get_receiver::<i32>().unwrap();

        tx1.send(4711);
        tx2.send(4951);
        assert_eq!(4711, rx1.receive());
        assert_eq!(4951, rx1.receive())       
    }
}
