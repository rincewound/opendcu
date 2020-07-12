
#[derive(Copy, Clone, Debug)]
pub enum ModuleCapability
{
    Inputs(u32),            // Number of digital ins provided by this module
    Outputs(u32),           // Number of digital outs provided by this module
    AccessPoints(u32),      // Number of access points provided by this module
    _KeypadEntry(u32),       // Number of keypad instances provided by this module
    //_Whitelist,
    // VirtualNetwork,      // At some point!
}


#[derive(Clone)]
pub struct ModuleCapabilityAdvertisement
{
    pub module_id: u32,
    pub caps: Vec<ModuleCapability>
}

pub struct ModCapAggregator
{
    entries: Vec<u32>,
    locked: bool
}

impl ModCapAggregator
{
    pub fn new() -> Self { Self { entries: Vec::new(), locked: false } }


    pub fn add_message(&mut self, module_id: u32, num_entries: u32) 
    {
        if self.locked
        {
            panic!("Cannot add modcap to already build aggregator.")
        }

        for index in module_id..module_id + num_entries
        {
            if self.entries.binary_search(&index).is_ok()
            {
                panic!("Cannot use the same module id/sud id twice.")
            }

            self.entries.push(index);
        }

    }

    pub fn build(&mut self) 
    {
        self.locked = true;
    }

    pub fn sud_to_logical_id(&self, sud: u32) -> Result<u32, ()> 
    {
        if let Ok(id) =self.entries.binary_search(&sud)
        {
            return Ok(id as u32)
        }
        return Err(())
    }

    pub fn logical_id_to_sud(&self, logical_id: u32) -> Result<u32, ()>
    {
        if self.entries.len() <= logical_id as usize
        {
            return Err(())
        }
        Ok(self.entries[logical_id as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn can_add_message()
    {   
        let mut a = ModCapAggregator::new();
        a.add_message(0x4711, 5);
        // Nobody panicked, all good!
        assert!(true);
    }

    #[test]
    #[should_panic]
    pub fn will_panic_if_message_is_added_after_build()
    {   
        let mut a = ModCapAggregator::new();        
        a.add_message(0x4711, 10 );
        a.build();
        a.add_message(0x4721, 10 );
    }

    #[test]
    #[should_panic]
    pub fn will_panic_if_sees_same_id_twice()
    {   
        let mut a = ModCapAggregator::new();
        a.add_message(0x4711, 10);
        a.add_message(0x4711, 5);
    }

    #[test]
    pub fn will_yield_correct_module_id()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(0x04000000, 10);
        a.add_message(0x05000000, 5);
        a.build();

        let result = a.logical_id_to_sud(13).unwrap();
        assert_eq!(result, 0x05000003);

        let result2 = a.logical_id_to_sud(2).unwrap();
        assert_eq!(result2, 0x04000002);
    }

    #[test]
    pub fn will_return_err_if_id_is_unknown()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(0x04000000, 10);
        a.add_message(0x05000000, 5);
        a.build();

        let result = a.logical_id_to_sud(455);
        assert!(result.is_err())
    }

    #[test]
    pub fn can_convert_sud_to_logical()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(0x04000000, 10);
        a.add_message(0x05000000, 5);
        a.build();

        let result = a.sud_to_logical_id(0x04000003).unwrap();
        assert_eq!(result, 3);
        let result = a.sud_to_logical_id(0x05000003).unwrap();
        assert_eq!(result, 13);
    }

    #[test]
    pub fn yields_err_on_bad_sud()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(0x04000000, 10);
        a.add_message(0x05000000, 5);
        a.build();

        let result = a.sud_to_logical_id(0x07000003);
        assert!(result.is_err());
    }

}
