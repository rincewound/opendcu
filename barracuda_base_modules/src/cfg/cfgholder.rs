use std::{collections::HashMap};

use barracuda_core::core::shareable::Shareable;


#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum FunctionType
{
    Put,
    //Get,
    Delete,
    Post
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum CfgError
{
    ResourceNotFound,
    ResourceEmpty
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct RouteKey
{
    func_ty: FunctionType,
    route: String
}

pub struct CfgHolder
{
    write_funcs: Shareable<HashMap<RouteKey, Box<dyn FnMut(Vec<u8>) ->() + Send>>>,
    read_funcs: Shareable<HashMap<String, Box<dyn FnMut() -> Vec<u8> + Send>>>
}

impl CfgHolder
{
    pub fn new() -> Self
    {
        CfgHolder{
            write_funcs: Shareable::new(HashMap::new()),
            read_funcs: Shareable::new(HashMap::new())
        }
    }

    fn make_key(&self, func: FunctionType, route: String) -> RouteKey
    {
        RouteKey {
            func_ty: func,
            route
        }
    }

    pub fn register_handler<F: 'static>(&mut self, functy: FunctionType, route: String, func: F )
    where F: FnMut(Vec<u8>) ->() + Send
    {
        let key = self.make_key(functy, route);
        self.write_funcs.lock()
                      .insert(key , Box::new(func));
    }

    pub fn register_read_handler<F: 'static>(&mut self, route: String, func: F )
    where F: FnMut() -> Vec<u8> + Send
    {
        self.read_funcs.lock()
                       .insert(route, Box::new(func));
    }

    fn do_action(&mut self, action: FunctionType, route: String, data: Vec<u8>)
    {
        // The trouble: func in this form is not copyable, so we have to first move it out of
        // the dict   
        let the_key = self.make_key(action, route);
        let item = self.write_funcs.lock()                                                             
                                                             .remove_entry(&the_key); 
        if item.is_none()
        {
            return;
        }
        let mut func = item.unwrap().1;
        func(data);
        // and put it back afterwards    
        self.write_funcs.lock()
                      .insert(the_key, func);
        // Crappily it seems to be impossible to use an Arc here, as calling the damn function would
        // require moving it out of the Arc as well.
    }

    pub fn do_put(&mut self, route: String, data: Vec<u8>)
    {     
        self.do_action(FunctionType::Put, route, data);
    }

    pub fn do_post(&mut self, route: String, data: Vec<u8>)
    {     
        self.do_action(FunctionType::Post, route, data);
    }

    pub fn do_delete(&mut self, route: String, data: Vec<u8>)
    {     
        self.do_action(FunctionType::Delete, route, data);
    }

    pub fn do_get(&mut self, route: String) -> Result<Vec<u8>, CfgError>
    {
        let item = self.read_funcs.lock().remove_entry(&route);
        if item.is_none()
        {
            return Err(CfgError::ResourceNotFound);
        }

        let mut func = item.unwrap().1;
        let result = func();
        self.read_funcs.lock().insert(route, func);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::cfg::cfgholder::*;
    use std::{sync::{Mutex, Arc}, cell::{RefCell}};
    
     #[test]
     pub fn put_triggers_correct_function()
     {
         let mut hdl = CfgHolder::new();
         let tmp = Arc::new(Mutex::new(RefCell::new(false)));
         let tmpclone = tmp.clone();
         hdl.register_handler(FunctionType::Put, "cfg/foo".to_string(), move |_t: Vec<u8>| {
             let m2 = tmpclone.clone();
             *(m2.lock().unwrap().borrow_mut()) = true;
             drop(m2);
         });
         hdl.do_put("cfg/foo".to_string(), Vec::from("{val:true}".as_bytes()));
         
         let result = *tmp.lock().unwrap().borrow_mut();
         assert_eq!(true, result)
     }

     #[test]
     pub fn put_does_not_trigger_if_registered_function_has_different_method()
     {
         let mut hdl = CfgHolder::new();
         let tmp = Arc::new(Mutex::new(RefCell::new(false)));
         let tmpclone = tmp.clone();
         hdl.register_handler(FunctionType::Post, "cfg/foo".to_string(), move |_t: Vec<u8>| {
            let m2 = tmpclone.clone();
            *(m2.lock().unwrap().borrow_mut()) = true;
             drop(m2);
         });
         hdl.do_put("cfg/foo".to_string(), Vec::from("{val:true}".as_bytes()));
         
         let result = *tmp.lock().unwrap().borrow_mut();
         assert_eq!(false, result)
     }

     #[test]
     pub fn put_triggers_a_post_triggers_b()
     {
         let mut hdl = CfgHolder::new();
         let tmp =  Arc::new(Mutex::new(RefCell::new(1)));
         let tmpclone = tmp.clone();
         hdl.register_handler(FunctionType::Put, "cfg/foo".to_string(), move |_t: Vec<u8>| {
            let m2 = tmpclone.clone();
            *(m2.lock().unwrap().borrow_mut()) = 2;
             drop(m2);
         });

         let secondclone = tmp.clone();
         hdl.register_handler(FunctionType::Post, "cfg/foo".to_string(), move |_t: Vec<u8>| {
            let m2 = secondclone.clone();
            *(m2.lock().unwrap().borrow_mut()) = 3;
            drop(m2);
        });

         hdl.do_put("cfg/foo".to_string(), Vec::from("{val:true}".as_bytes()));
         
         let result = *tmp.lock().unwrap().borrow();
         assert_eq!(2, result);
         drop(result);
         hdl.do_post("cfg/foo".to_string(), Vec::from("{val:true}".as_bytes()));
         let result2 = *tmp.lock().unwrap().borrow();
         assert_eq!(3, result2);
     }

     #[test]
     pub fn put_does_not_fail_if_unknown_route_is_triggered()
     {
         let mut hdl = CfgHolder::new();
         hdl.do_put("cfg/bar".to_string(), Vec::from("{val:true}".as_bytes()));
         // Nothing to check, we just want to make sure this does not panic.
     }

     #[test]
     pub fn get_returns_value()
     {
        let mut hdl = CfgHolder::new();
        hdl.register_read_handler("cfg/foo".to_string(), move|| {
            return vec![1,2,3,4]
        });

        let result = hdl.do_get("cfg/foo".to_string()).expect("Not ok!");
        assert_eq!(result, vec![1,2,3,4])
     }

     #[test]
     pub fn get_yields_not_found_for_bad_route()
     {
        let mut hdl = CfgHolder::new();
        let result  = hdl.do_get("some/route".to_string()).expect_err("Not an error");
        assert_eq!(result, CfgError::ResourceNotFound);
     }
}

