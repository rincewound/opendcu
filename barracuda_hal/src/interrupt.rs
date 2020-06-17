pub trait Interrupt
{
    fn wait(&self);
    fn wait_timeout(&self,timeout_ms: u32) -> bool;
}