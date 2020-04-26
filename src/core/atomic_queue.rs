

use std::{collections::VecDeque, sync::{Arc, Mutex}, cell::Cell};
use super::event::{DataEvent, Event};


pub struct AtomicQueue<T: Clone>
{
    data: Mutex<Cell<VecDeque<T>>>,
    data_trigger: Mutex<Cell<(Option<Arc<DataEvent<u32>>>, u32)>>,
    evt: Event
}

impl<T: Clone> AtomicQueue<T> {
    pub fn new() -> Self 
    { 
        Self 
        { 
            data: Mutex::new(Cell::new(VecDeque::new())), 
            data_trigger: Mutex::new(Cell::new((None, 0))),
            evt: Event::new(),
        } 
    }

    pub fn push(&self, data: T)
    {
        let mut d = self.data.lock().unwrap();
        d.get_mut().push_back(data);
        self.evt.trigger();
        let mut trg = self.data_trigger
                                                            .lock()
                                                            .unwrap();
        let dat = trg.get_mut();
        if let Some(e) = &dat.0
        {
            e.trigger(dat.1);
        }
    }

    pub fn wait_data(&self)
    {        
        if self.len() != 0
        {
            return;
        }
        self.evt.wait();
    }

    pub fn wait_with_timeout(&self, milliseconds: u64) -> bool
    {   
        if self.len() != 0
        {
            return true;
        }
        return self.evt.wait_with_timeout(milliseconds);       
    }

    pub fn set_data_trigger(&self, evt: Arc<DataEvent<u32>>, trigger_data: u32)
    {
        if self.len() != 0
        {
            evt.trigger(trigger_data)            
        }
        let mut trg = self.data_trigger
                                                        .lock()
                                                        .unwrap();
        let dat = trg.get_mut();
        dat.0 = Some(evt);
        dat.1 = trigger_data;
    }

    pub fn pop(&self) -> Option<T>
    {
        return self.data.lock().unwrap().get_mut().pop_front();
    }

    pub fn len(&self) -> usize
    {
        return self.data.lock().unwrap().get_mut().len();
    }
}



#[cfg(test)]
mod tests {
     use crate::core::atomic_queue::*;

     #[test]
     fn can_push_data()
     {
         let q = AtomicQueue::new();
         q.push(32);         
     }

     #[test]
     fn can_pop_data()
     {
         let q = AtomicQueue::new();
         q.push(32);   
         let d = q.pop().unwrap();
         assert_eq!(32, d); 
     }

     #[test]
     fn returns_trigger_data()
     {        
        let mut q  = AtomicQueue::new();    
        let e = Arc::new(DataEvent::<u32>::new()); 

        q.set_data_trigger(e.clone(), 7);
        q.push(10);   
        let trig_data = e.wait();
        assert_eq!(7, trig_data);
     }
    
}



