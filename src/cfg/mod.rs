use std::sync::{Mutex, Arc};
use crate::core::shareable::Shareable;

pub mod REST;

// use serde::{Serialize, Deserialize};
// use serde_json::{Result, Value};

/*

We need a macro that calls Serde to convert a given
request body to a struct of a given kind, i.e:

fn Convert(r: Request): Option<T>
{

}

and then Calls the handlerfunction,
which is an FnOnce using the converted
value

*/
pub mod cfgholder;

#[derive(Clone)]
pub enum ConfigMessage
{
    RegisterHandlers(Shareable<cfgholder::CfgHolder>)
}


macro_rules! Handler {
    ($func: expr) => {
        (move |req : Vec<u8>| { 
            let e = cfg::convert_data(req); 
            if e.is_some() 
            {
                $func(e.unwrap())
            } 
        })
    };
}

/*

The rubbish bit here ist, that this introduces a rouille dependency
for all components, even if we - at some point, want to
have use a different CFG module.
*/
pub fn convert_data<T: for<'de> serde::Deserialize<'de>>(r: Vec<u8>) -> Option<T>
{
    //let rdr = r.data().unwrap();
    let someval = serde_json::from_slice(&r[..]);
    if let Ok(data) = someval {
        return Some(data)
    }
    None
}
