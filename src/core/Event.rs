use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;

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
            let result = cvar.wait_timeout(started, Duration::from_millis(10)).unwrap();
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