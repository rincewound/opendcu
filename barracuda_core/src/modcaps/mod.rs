
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

pub enum ModuleCapabilityType
{
    Inputs,            // Number of digital ins provided by this module
    Outputs,           // Number of digital outs provided by this module
    AccessPoints,      // Number of access points provided by this module
    _KeypadEntry,       // Number of keypad instances provided by this module
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
    inputs: Vec<u32>,
    outputs: Vec<u32>,
    accessPoints: Vec<u32>,
    keypads: Vec<u32>,
    locked: bool
}

impl ModCapAggregator
{
    pub fn new() -> Self
    {
        Self { 
            inputs: vec![],
            outputs: vec![],
            accessPoints: vec![],
            keypads: vec![],
            locked: false
        }
    }

    pub fn aggregate(&mut self, message_receiver: &crate::core::broadcast_channel::GenericReceiver<ModuleCapabilityAdvertisement>)
    {
        while let Some(modcap_message) = message_receiver.receive_with_timeout(0)
        {
            self.add_message(modcap_message)
        }
        self.build();
    }

    pub fn get_num_entries(&self, cap: ModuleCapabilityType) -> usize
    {
        let source_list;
        match cap
        {
            ModuleCapabilityType::Inputs => source_list = &self.inputs,
            ModuleCapabilityType::Outputs => source_list = &self.outputs,
            ModuleCapabilityType::AccessPoints => source_list = &self.accessPoints,
            ModuleCapabilityType::_KeypadEntry => source_list = &self.inputs
        }
        source_list.len()
    }

    pub fn add_message(&mut self, message: ModuleCapabilityAdvertisement) 
    {
        if self.locked
        {
            panic!("Cannot add modcap to already build aggregator.")
        }

        for cap in message.caps.iter()
        {
            match cap
            {
                ModuleCapability::Inputs(num_in) => Self::add_cap(&mut self.inputs, message.module_id, *num_in),
                ModuleCapability::Outputs(num_out) => Self::add_cap(&mut self.outputs, message.module_id, *num_out),
                ModuleCapability::AccessPoints(num_ap) => Self::add_cap(&mut self.accessPoints, message.module_id, *num_ap),
                ModuleCapability::_KeypadEntry(num_kp) => Self::add_cap(&mut self.inputs, message.module_id, *num_kp),
            }
        }
    }

    fn add_cap(dest: &mut Vec<u32>, module_id: u32, num_entries: u32)
    {
        
        for index in module_id..module_id + num_entries
        {
            if dest.binary_search(&index).is_ok()
            {
                panic!("Cannot use the same module id/sud id twice.")
            }

            dest.push(index);
        }
    }

    pub fn build(&mut self) 
    {
        self.inputs.sort_unstable_by(|a,b| a.partial_cmp(b).unwrap());
        self.outputs.sort_unstable_by(|a,b| a.partial_cmp(b).unwrap());
        self.keypads.sort_unstable_by(|a,b| a.partial_cmp(b).unwrap());
        self.accessPoints.sort_unstable_by(|a,b| a.partial_cmp(b).unwrap());
        self.locked = true;

    }

    pub fn sud_to_logical_id(&self, sud: u32, cap: ModuleCapabilityType) -> Result<u32, ()> 
    {
        if !self.locked
        {
            panic!("Cannot convert SUD if Aggregator is not locked.")
        }

        let search_list;
        match cap
        {
            ModuleCapabilityType::Inputs => search_list = &self.inputs,
            ModuleCapabilityType::Outputs => search_list = &self.outputs,
            ModuleCapabilityType::AccessPoints => search_list = &self.accessPoints,
            ModuleCapabilityType::_KeypadEntry => search_list = &self.inputs
        }

        if let Ok(id) = search_list.binary_search(&sud)
        {
            return Ok(id as u32)
        }

        return Err(())
    }

    pub fn logical_id_to_sud(&self, logical_id: u32, cap: ModuleCapabilityType) -> Result<u32, ()>
    {
        if !self.locked
        {
            panic!("Cannot convert id if Aggregator is not locked.")
        }

        let search_list;
        match cap
        {
            ModuleCapabilityType::Inputs => search_list = &self.inputs,
            ModuleCapabilityType::Outputs => search_list =  &self.outputs,
            ModuleCapabilityType::AccessPoints => search_list = &self.accessPoints,
            ModuleCapabilityType::_KeypadEntry => search_list = &self.inputs
        }

        if search_list.len() <= logical_id as usize
        {
            return Err(())
        }

        Ok(search_list[logical_id as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message(mod_id: u32, cap: ModuleCapability) -> ModuleCapabilityAdvertisement
    {
        return ModuleCapabilityAdvertisement{module_id: mod_id, caps: vec![cap]};
    }

    #[test]
    pub fn can_add_message()
    {   
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x4722, ModuleCapability::Inputs(5)));
        // Nobody panicked, all good!
        assert!(true);
    }

    #[test]
    #[should_panic]
    pub fn will_panic_if_message_is_added_after_build()
    {   
        let mut a = ModCapAggregator::new();        
        a.add_message(make_message(0x4711, ModuleCapability::Inputs(5)));
        a.build();
        a.add_message(make_message(0x4721, ModuleCapability::Inputs(10)));
    }

    #[test]
    #[should_panic]
    pub fn will_panic_if_sees_same_id_twice()
    {   
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x4711, ModuleCapability::Inputs(5)));
        a.add_message(make_message(0x4711, ModuleCapability::Inputs(5)));
    }

    #[test]
    #[should_panic]
    pub fn sud_to_logical_will_panic_if_build_was_not_called()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x4711, ModuleCapability::Inputs(10)));
        let _ = a.sud_to_logical_id(0x47110011, ModuleCapabilityType::Inputs);
    }

    #[test]
    #[should_panic]
    pub fn logical_to_sud_will_panic_if_build_was_not_called()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x4711, ModuleCapability::Inputs(10)));
        let _ = a.logical_id_to_sud(5, ModuleCapabilityType::Inputs);
    }

    #[test]
    pub fn will_yield_correct_module_id()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x04000000, ModuleCapability::Inputs(10)));
        a.add_message(make_message(0x05000000, ModuleCapability::Inputs(5)));
        a.build();

        let result = a.logical_id_to_sud(13, ModuleCapabilityType::Inputs).unwrap();
        assert_eq!(result, 0x05000003);

        let result2 = a.logical_id_to_sud(2, ModuleCapabilityType::Inputs).unwrap();
        assert_eq!(result2, 0x04000002);
    }

    #[test]
    pub fn will_return_err_if_id_is_unknown()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x04000000, ModuleCapability::Inputs(10)));
        a.add_message(make_message(0x05000000, ModuleCapability::Inputs(5)));
        a.build();

        let result = a.logical_id_to_sud(455, ModuleCapabilityType::Inputs);
        assert!(result.is_err())
    }

    #[test]
    pub fn can_convert_sud_to_logical()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x04000000, ModuleCapability::Inputs(10)));
        a.add_message(make_message(0x05000000, ModuleCapability::Inputs(5)));
        a.build();

        let result = a.sud_to_logical_id(0x04000003, ModuleCapabilityType::Inputs).unwrap();
        assert_eq!(result, 3);
        let result = a.sud_to_logical_id(0x05000003, ModuleCapabilityType::Inputs).unwrap();
        assert_eq!(result, 13);
    }

    #[test]
    pub fn yields_err_on_bad_sud()
    {
        let mut a = ModCapAggregator::new();
        a.add_message(make_message(0x04000000, ModuleCapability::Inputs(10)));
        a.add_message(make_message(0x05000000, ModuleCapability::Inputs(5)));
        a.build();

        let result = a.sud_to_logical_id(0x07000003, ModuleCapabilityType::Inputs);
        assert!(result.is_err());
    }

}
