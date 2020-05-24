

use super::{shareable::Shareable, event::Event};
use std::{time::{Instant, Duration}, thread, sync::{Weak, Arc}};

struct TimerEntry
{
    callback: Option<Box<dyn FnOnce() -> () + Send>>,
    due_time: Instant,
    remove: bool,
    guard: Weak<bool>
}

pub struct Timer where
{
    scheduled_calls: Shareable<Vec<TimerEntry>>,
    wait_event: Event,
    terminate: Shareable<bool>
}

impl Timer
{
    pub fn new() -> Arc<Self>
    {
        //let mut chm = ChannelManager::new();
        let result = Arc::new(Timer {
            scheduled_calls:    Shareable::new(Vec::new()),
            wait_event:         Event::new(),
            terminate:          Shareable::new(false)         
        });
        Timer::start(result.clone());
        return result;
    }

    fn start(tm: Arc<Timer>)
    {
        let timer = tm.clone();
        thread::spawn(move|| {             
           timer.thread_func();              
        });
    }

    pub fn stop(&self)
    {
        let mut term = self.terminate.lock();
        *term = true;
        self.wait_event.trigger();
    }

    pub fn schedule(&self, callback: Box<dyn FnOnce() -> () + Send>, delay: u64) -> Arc<bool>
    {
        let mut calls = self.scheduled_calls.lock();
        let due_time = Instant::now() + Duration::from_millis(delay);
        let guard = Arc::new(false);
        calls.push(TimerEntry{
            callback: Some(callback), 
            due_time: due_time, 
            remove: false,
            guard: Arc::downgrade(&guard)});
        self.wait_event.trigger();
        return guard;
    }

    fn get_min_remaining_timeout(&self) -> Instant
    {
        let calls = self.scheduled_calls.lock();
        let min_remaining = calls.iter()
                                                         .min_by_key(|x| x.due_time);

        if let Some(remaining) = min_remaining
        {
            return remaining.due_time;
        }
        else
        {   
            // Nothing valid found, we either have no entries left or
            // the first entry found is already in the past.
            // In the first case we want to trigger right away, in the latter
            // case we'll wait for as long as necessary.
            if calls.len() != 0
            {
                return Instant::now();
            }
            else
            {
                return Instant::now() + Duration::from_secs(10);
            }
        }
    }

    fn trigger_active_entries(&self)
    {
        let mut calls = self.scheduled_calls.lock();

        calls.iter_mut().for_each(|elem|{
            let now = Instant::now();
            let due_time = elem.due_time;
            if now >= due_time
            {  
                if elem.guard.upgrade().is_some()
                {
                    let tmp = elem.callback.take();
                    (tmp.unwrap())();
                }
                elem.remove = true;
            }
        });

        // all the entries that have been called have their remove flag
        // set -> we only keep those, that have not been called yet.
        calls.retain(|x| !x.remove);

    }

    fn thread_func(&self)
    {
        loop {
            let min_remaining = self.get_min_remaining_timeout();            
            let due_ticks = min_remaining.checked_duration_since(Instant::now());

            let wait_timeout: u64;
            if let Some(ticks) = due_ticks
            {
                wait_timeout = ticks.as_millis() as u64;
            }
            else
            {
                // Nothing valid found, we either have no entries left or
                // the first entry found is already in the past.
                wait_timeout = 0;
            }

            if self.wait_event.wait_with_timeout(wait_timeout)
            {
                // event was triggered -> this means someone has either
                // added a new element to the remaining calls or
                // the d'tor was called.
                let term = self.terminate.lock();
                if *term == true
                {
                    return;
                }
            }
            else
            {
                // nothing received - we got a timeout, which means we should
                // attempt to trigger 
                self.trigger_active_entries();     
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Timer;
    use crate::core::shareable::Shareable;
    use std::{time::Duration, thread::sleep};

    #[test]
    fn can_create_timer()
    {
        let t = Timer::new();
        t.stop();
    }

    #[test]
    fn timer_calls_callback()
    {
        let flag = Shareable::new(false);
        let t = Timer::new();
        let movable_clone = flag.clone();
        let guard = t.schedule(Box::new(move || {
            let mut flag_access = movable_clone.lock();
            *flag_access = true;
        }), 50);

        sleep(Duration::from_millis(100));
        drop(guard);
        assert!(*flag.lock() == true); 
    }

    #[test]
    fn can_abort_scheduled_call()
    {
        let flag = Shareable::new(false);
        let t = Timer::new();
        let movable_clone = flag.clone();
        let guard = t.schedule(Box::new(move || {
            let mut flag_access = movable_clone.lock();
            *flag_access = true;
        }), 50);
        
        drop(guard);

        sleep(Duration::from_millis(100));

        assert!(*flag.lock() == false); 
    }
}