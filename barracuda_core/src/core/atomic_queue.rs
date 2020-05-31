

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
        self.do_data_trigger();
    }

    fn do_data_trigger(&self)
    {
        let mut trg = self.data_trigger
                                                            .lock()
                                                            .unwrap();
        let dat = trg.get_mut();

        let evt = dat.0.take();
        if let Some(e) = evt
        {  
            //println!("Trigger event {}", e.name);  
            e.trigger(dat.1);
            dat.0 = Some(e);
        }        
    }

    pub fn wait_data(&self)
    {        
        if self.len() != 0
        {
            self.evt.reset();
            return;
        }
        self.evt.wait();
    }

    pub fn wait_with_timeout(&self, milliseconds: u64) -> bool
    {          
        if self.len() != 0
        {
            self.evt.reset();
            return true;
        } 
        return self.evt.wait_with_timeout(milliseconds);       
    }

    pub fn set_data_trigger(&self, evt: Arc<DataEvent<u32>>, trigger_data: u32)
    {
        let mut trg = self.data_trigger
                                                        .lock()
                                                        .unwrap();
        let dat = trg.get_mut();

        // let mut prev_name = String::from("None");
        // if dat.0.is_some()
        // {
        //     let tmp = dat.0.take().unwrap();
        //     prev_name = tmp.name.clone();
        //     dat.0 = Some(tmp);
        // }

        //println!("Dataevent set to {}, was {}", evt.name, prev_name);
        dat.0 = Some(evt);
        dat.1 = trigger_data;
        drop(trg);
        
        if self.len() != 0
        {
            self.do_data_trigger();
        }
    }

    pub fn pop(&self) -> Option<T>
    {
        // Rubbish: This never resets the data event, so 
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
        let q  = AtomicQueue::new();    
        let e = Arc::new(DataEvent::<u32>::new("Foo".to_string())); 

        q.set_data_trigger(e.clone(), 7);
        q.push(10);   
        let trig_data = e.wait();
        assert_eq!(7, trig_data);
     }

     #[test]
     fn can_trigger_twice()
     {
        let q  = AtomicQueue::new();    
        let e = Arc::new(DataEvent::<u32>::new("Foo".to_string())); 

        q.set_data_trigger(e.clone(), 7);
        q.push(10);   
        let trig_data = e.wait();
        assert_eq!(7, trig_data);  
        let e2 = Arc::new(DataEvent::<u32>::new("Foo".to_string())); 
        q.set_data_trigger(e2.clone(), 5);   
        let trig_data = e2.wait();
        assert_eq!(5, trig_data);  
     }
    
}



