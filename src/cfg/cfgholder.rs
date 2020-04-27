use std::{sync::Arc, collections::HashMap};

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
enum FunctionType
{
    Put,
    //Get,
    Delete,
    Post
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct RouteKey
{
    funcTy: FunctionType,
    route: String
}

pub struct CfgHolder<ReaderTy>
{
    put_funcs: HashMap<RouteKey, Box<dyn FnMut(ReaderTy) ->()>>
}

impl <'a, ReaderTy: std::io::Read> CfgHolder<ReaderTy>
{
    pub fn new() -> Self
    {
        CfgHolder{
            put_funcs: HashMap::new()
        }
    }

    fn make_key(&self, func: FunctionType, route: String) -> RouteKey
    {
        RouteKey {
            funcTy: func,
            route
        }
    }

    pub fn register_handler<F: 'static>(&mut self, functy: FunctionType, route: String, Func: F )
    where F: 'a + FnMut(ReaderTy) ->()
    {
        let key = self.make_key(functy, route);
        self.put_funcs.insert(key , Box::new(Func));
    }

    fn do_action(&mut self, action: FunctionType, route: String, data: ReaderTy)
    {
                // The trouble: func in this form is not copyable, so we have to first move it out of
        // the dict   
        let theKey = self.make_key(action, route);
        let item = self.put_funcs.remove_entry(&theKey); //.unwrap().1;
        if item.is_none()
        {
            return;
        }
        let mut func = item.unwrap().1;
        func(data);
        // and put it back afterwards    
        self.put_funcs.insert(theKey, func);
        // Crappily it seems to be impossible to use an Arc here, as calling the damn function would
        // require moving it out of the Arc as well.
    }

    pub fn do_put(&mut self, route: String, data: ReaderTy)
    {     
        self.do_action(FunctionType::Put, route, data);
    }

    pub fn do_post(&mut self, route: String, data: ReaderTy)
    {     
        self.do_action(FunctionType::Post, route, data);
    }

    pub fn do_delete(&mut self, route: String, data: ReaderTy)
    {     
        self.do_action(FunctionType::Delete, route, data);
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
         hdl.register_handler(FunctionType::Put, "cfg/foo".to_string(), move |_t: &[u8]| {
             let mut m2 = tmpclone.borrow_mut();
             *m2 = true;
             drop(m2);
         });
         hdl.do_put("cfg/foo".to_string(), "{val:true}".as_bytes());
         
         let result = *tmp.borrow();
         assert_eq!(true, result)
     }

     #[test]
     fn put_does_not_trigger_if_registered_function_has_different_method()
     {
         let mut hdl = CfgHolder::new();
         let tmp = Arc::new(RefCell::new(false));
         let tmpclone = tmp.clone();
         hdl.register_handler(FunctionType::Post, "cfg/foo".to_string(), move |_t: &[u8]| {
             let mut m2 = tmpclone.borrow_mut();
             *m2 = true;
             drop(m2);
         });
         hdl.do_put("cfg/foo".to_string(), "{val:true}".as_bytes());
         
         let result = *tmp.borrow();
         assert_eq!(false, result)
     }

     #[test]
     fn put_triggers_a_post_triggers_b()
     {
         let mut hdl = CfgHolder::new();
         let tmp = Arc::new(RefCell::new(1));
         let tmpclone = tmp.clone();
         hdl.register_handler(FunctionType::Put, "cfg/foo".to_string(), move |_t: &[u8]| {
             let mut m2 = tmpclone.borrow_mut();
             *m2 = 2;
             drop(m2);
         });

         let secondclone = tmp.clone();
         hdl.register_handler(FunctionType::Post, "cfg/foo".to_string(), move |_t: &[u8]| {
            let mut m2 = secondclone.borrow_mut();
            *m2 = 3;
            drop(m2);
        });

         hdl.do_put("cfg/foo".to_string(), "{val:true}".as_bytes());
         
         let result = *tmp.borrow();
         assert_eq!(2, result);
         drop(result);
         hdl.do_post("cfg/foo".to_string(), "{val:true}".as_bytes());
         let result2 = *tmp.borrow();
         assert_eq!(3, result2);
     }

     #[test]
     fn put_does_not_fail_if_unknown_route_is_triggered()
     {
         let mut hdl = CfgHolder::new();
         hdl.do_put("cfg/bar".to_string(), "{val:true}".as_bytes());
         // Nothing to check, we just want to make sure this does not panic.
     }
}

