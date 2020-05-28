use std::sync::{Arc, Mutex, MutexGuard};

/// # A piece of shareable state
/// This struct wraps the common
/// Arc<Mutex<Data>> pattern used
/// for internal mutability for 
/// easier usage.
pub struct Shareable<T>
{
    data: Arc<Mutex<T>>
}

impl<T> Shareable<T>
{
    pub fn new(data: T) -> Self
    {
        Shareable
        {
            data: Arc::new(Mutex::new(data))
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T>
    {
        return self.data.lock().unwrap();
    }

}

impl <T> Clone for Shareable<T>
{
    fn clone(&self) -> Self {
        Shareable
        {
            data: self.data.clone()
        }
    }
    
}