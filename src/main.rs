
#[macro_use]
extern crate rouille;

//#[macro_use]
//extern crate diesel;

extern crate serde;

#[macro_use]
mod core;
#[macro_use]
mod cfg;

mod trace;
mod acm;
mod arm;
mod sig;
mod dcm;
mod modcaps;
mod webserver;
mod io;


// example how to simplify the launching threads
struct Test {}

impl Test
{
    pub fn new() -> Self
    {
        Test {}
    }
    pub fn run(self)
    {
        println!("run");
    }
}

struct Test1 {}

impl Test1
{
    pub fn new() -> Self
    {
        Test1 {}
    }
    pub fn run(self)
    {
        println!("run 1");
    }
}

macro_rules! test
{
    ($head:ident, $($test1: ident),+) =>
    {
        let tracer = "tracer";
        let test = $head::new();
        test.run();
        test!($($test1),+);
    };
    ($head:ident) =>
    {
        let test = $head::new();
        test.run();
    }
}

fn main() {
    // run
    // test!(Test, Test1);

    // Note: Launch never returns!
    launch!(trace::launch,
            // launch_thread!(create::cfg::REST::ConfigRest, )
            crate::cfg::REST::launch,
            crate::acm::generic_whitelist::launch::<acm::generic_whitelist::whitelist::JsonEntryProvider>,
            crate::arm::console_input::launch,
            crate::webserver::launch,
            crate::io::launch,
            crate::dcm::trivial::launch);
}