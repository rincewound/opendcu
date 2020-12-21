
use crate::core::shareable::Shareable;

pub mod rest;

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

#[macro_export]
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

#[macro_export]
macro_rules! ReadDataHandler {
    ($func: expr) => {
        (move |req : Vec<u8>| { 
            let result =  $func(e.unwrap())
            return cfg::serialize_data(result); 
        })
    };
}

pub fn convert_data<T: for<'de> serde::Deserialize<'de>>(r: Vec<u8>) -> Option<T>
{
    let someval = serde_json::from_slice(&r[..]);
    if let Ok(data) = someval {
        return Some(data)
    }
    None
}

pub fn serialize_data<T: for<'de> serde::Serialize>(data: T) -> Option<Vec<u8>>
{
    let someval = serde_json::to_vec(&data);
    if let Ok(data) = someval {
        return Some(data)
    }
    None
}
