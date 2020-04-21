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

macro_rules! Handler {
    ($func: expr) => {
        (|req| { 
            let e = cfg::convert_data(req); 
            if e.is_some() 
            {
                $func(e.unwrap())
            } 
        })
    };
}

pub fn convert_data<T: for<'de> serde::Deserialize<'de>>(r: rouille::Request) -> Option<T>
{
    let rdr = r.data().unwrap();
    let someval = serde_json::from_reader(rdr);
    if let Ok(data) = someval {
        return Some(data)
    }
    None
}
