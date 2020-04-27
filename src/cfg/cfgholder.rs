use std::{sync::Arc, collections::HashMap};



pub struct CfgHolder<ReaderTy>
{
    put_funcs: HashMap<String, Box<dyn FnMut(ReaderTy) ->()>>
}

impl <'a, ReaderTy: std::io::Read> CfgHolder<ReaderTy>
{
    pub fn new() -> Self
    {
        CfgHolder{
            put_funcs: HashMap::new()
        }
    }

    pub fn register_handler<F: 'static>(&mut self, route: String, Func: F )
    where F: 'a + FnMut(ReaderTy) ->()
    {
        self.put_funcs.insert(route, Box::new(Func));
    }

    pub fn do_put(&mut self, route: String, data: ReaderTy)
    {        
        let mut func = self.put_funcs.remove_entry(&route).unwrap().1;
        func(data);
        self.put_funcs.insert(route, func);
    }
}

#[cfg(test)]
mod tests {
    use crate::cfg::cfgholder::*;
    use std::{sync::Arc, cell::{RefCell}};
    
    #[macro_use]
    use crate::cfg;
     
     struct TestStruct
     {
         pub val: bool
     }

     #[test]
     fn put_triggers_correct_function()
     {
         let mut hdl = CfgHolder::new();
         let tmp = Arc::new(RefCell::new(false));
         let tmpclone = tmp.clone();
         hdl.register_handler("cfg/foo".to_string(), move |t: &[u8]| {
             let mut m2 = tmpclone.borrow_mut();
             *m2 = true;
             drop(m2);
         });
         hdl.do_put("cfg/foo".to_string(), "{val:true}".as_bytes());
         
         let result = *tmp.borrow();
         assert_eq!(true, result)
     }
}

