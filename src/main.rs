
mod core;
mod Trace;

use Trace::*;

fn main() {
    let mut chm = core::ChannelManager::ChannelManager::new();
    Trace::launch(&mut chm);

    let trace_tx = chm.get_sender::<Trace::trace_message>().unwrap();
    trace_tx.send(trace_message::from_str("..welcome to barracuda. Starting up."));


}
