use std::sync::{Arc, Mutex, Condvar};
use std::{cell::Cell, time::Duration};

pub struct Event{
    state: Arc<(Mutex<bool>, Condvar)>,
}

impl Event
{
    pub fn new() -> Self
    {
        Event{
            state: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub fn reset(&self)
    {
        let &(ref mtx, ref _cnd) = &*self.state;
        let mut guard = mtx.lock().unwrap();
        *guard = false;
    }

    pub fn wait(&self)
    {
        let &(ref mtx, ref cnd) = &*self.state;
        let mut guard = mtx.lock().unwrap();
        while !*guard {
            guard = cnd.wait(guard).unwrap();
            
        }
        *guard = false;
    }

    pub fn wait_with_timeout(&self, millis: u64) -> bool
    {
        let &(ref lock, ref cvar) = &*self.state.clone();
        let mut started = lock.lock().unwrap();  
        
        if *started == true
        {
            *started = false;
            return true;
        }
        
        loop 
        {
            /*
                Trouble: This implementation will fail, if cvar was triggered beforehand. In this case we
                always use up the t/o, although we might actually still return "not timed out" afterwards,
                (if started was set to true bevor wait_timeout was entered.)
            */

            // Let's put a timeout on the condvar's wait.
            let result = cvar.wait_timeout(started, Duration::from_millis(millis)).unwrap();
            
            started = result.0;
            if *started == true 
            {                
                *started = false;
                return true;
            }

            if result.1.timed_out()
            {
                return false;
            }

        }
    }

    pub fn trigger(&self)
    {
        let &(ref mtx, ref cvar) = &*(self.state.clone());
        let mut done = mtx.lock()
                          .unwrap();
        *done = true;
        cvar.notify_one();
    }
}

pub struct DataEvent <T: Copy+Sync>{
    evt: Event,
    data: Mutex<Cell<Option<T>>>,
    pub name: String
}

impl<T: Copy+Sync> DataEvent<T>
{
    pub fn new(name: String) -> Self
    {
        DataEvent{
            evt: Event::new(),
            data: Mutex::new(Cell::new(None)),
            name
        }
    }

    pub fn wait(&self) -> T
    {
        self.evt.wait();
        return self.data.lock()
                        .unwrap()
                        .take()
                        .unwrap();
    }

    #[allow(dead_code)]
    pub fn wait_with_timeout(&self, millis: u64) -> Option<T>
    {
        if self.evt.wait_with_timeout(millis)
        {
            return self.data.lock().unwrap().take();
        }

        return None;        
    }

    pub fn trigger(&self, data: T)
    {
        self.data.lock().unwrap().set(Some(data));
        self.evt.trigger();
    }
}

#[cfg(test)]
mod tests {
     use crate::core::event::*;
     use std::time::Instant;

     #[test]
     fn can_create_data_event()
     {
         let _ = DataEvent::<u32>::new("Foo".to_string());      
     }

     #[test]
     fn wait_yields_data_when_triggered()
     {
         let e = DataEvent::<u32>::new("Foo".to_string());      
         e.trigger(1048);
         assert_eq!(1048, e.wait())
     }

     #[test]
     fn event_resets_after_trigger()
     {
         let e =  Event::new();
         e.trigger();
         assert!(e.wait_with_timeout(50));
         assert!(!e.wait_with_timeout(10));
     }

     #[test]
     fn event_can_be_triggered_twice()
     {
         let e =  Event::new();
         e.trigger();
         assert!(e.wait_with_timeout(10));
         e.trigger();
         assert!(e.wait_with_timeout(10));
     }

     #[test]
     fn even_yield_failure_on_timeout()
     {
        let e =  Event::new();
        assert!(false == e.wait_with_timeout(10));        
     }

     #[test]
     fn trigger_before_wait_returns_immediately()
     {
        let e =  Event::new();
        e.trigger();        
        // This should return immediately!
        let now = Instant::now();
        assert!(e.wait_with_timeout(2000)); 
        assert!(now.elapsed().as_millis() < 1000);
     }
}

