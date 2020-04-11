use std::sync::{Arc, Mutex, Condvar};
use std::{cell::Cell, time::Duration};

pub struct Event{
    state: Arc<(Mutex<bool>, Condvar)>
}

impl Event
{
    pub fn new() -> Self
    {
        Event{
            state: Arc::new((Mutex::new(false), Condvar::new()))
        }
    }

    pub fn wait(&self)
    {
        let &(ref mtx, ref cnd) = &*self.state;
        let mut guard = mtx.lock().unwrap();
        while !*guard {
            guard = cnd.wait(guard).unwrap();
        }
    }

    pub fn wait_with_timeout(&self, millis: u64) -> bool
    {
        let &(ref lock, ref cvar) = &*self.state.clone();
        let mut started = lock.lock().unwrap();
        loop 
        {
            // Let's put a timeout on the condvar's wait.
            let result = cvar.wait_timeout(started, Duration::from_millis(millis)).unwrap();
            // 10 milliseconds have passed, or maybe the value changed!
            
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
        let &(ref mtx, _) = &*(self.state.clone());
        let mut done = mtx.lock()
                          .unwrap();
        *done = true;
    }
}

pub struct DataEvent <T: Copy+Sync>{
    state: Arc<(Mutex<bool>, Condvar)>,
    data: Mutex<Cell<Option<T>>>
}

impl<T: Copy+Sync> DataEvent<T>
{
    pub fn new() -> Self
    {
        DataEvent{
            state: Arc::new((Mutex::new(false), Condvar::new())),
            data: Mutex::new(Cell::new(None))
        }
    }

    pub fn wait(&self) -> T
    {
        let &(ref mtx, ref cnd) = &*self.state;
        let mut guard = mtx.lock().unwrap();
        while !*guard {
            guard = cnd.wait(guard).unwrap();
        }
        return self.data.lock()
                        .unwrap()
                        .take()
                        .unwrap();
    }

    pub fn wait_with_timeout(&self, millis: u64) -> Option<T>
    {
        let &(ref lock, ref cvar) = &*self.state.clone();
        let mut started = lock.lock().unwrap();
        loop 
        {
            // Let's put a timeout on the condvar's wait.
            let result = cvar.wait_timeout(started, Duration::from_millis(millis)).unwrap();
            // 10 milliseconds have passed, or maybe the value changed!
            
            started = result.0;
            if *started == true 
            {
                *started = false;
                return self.data.lock().unwrap().take();
            }

            if result.1.timed_out()
            {
                return None;
            }

        }
    }

    pub fn trigger(&self, data: T)
    {
        let &(ref mtx, _) = &*(self.state.clone());
        let mut done = mtx.lock()
                          .unwrap();
        self.data.lock().unwrap().set(Some(data));
        *done = true;
    }
}

#[cfg(test)]
mod tests {
     use crate::core::Event::*;

     #[test]
     fn can_create_data_event()
     {
         let e = DataEvent::<u32>::new();      
     }

     #[test]
     fn wait_yields_data_when_triggered()
     {
         let e = DataEvent::<u32>::new();      
         e.trigger(1048);
         assert_eq!(1048, e.wait())
     }

}

