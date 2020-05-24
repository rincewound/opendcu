

pub struct json_storage<ValueType>
{
    data: Vec<ValueType>,
    file_name: String

}

impl json_storage<ValueType>
{
    pub fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<ValueType> 
    {
        None
    }

    pub fn put_entry(&mut self, entry: ValueType) 
    { 
        //self.entries.retain(|x| x.identification_token_id.cmp(&entry.identification_token_id) != Ordering::Equal);
        self.data.push(entry);
        self.update_storage();
    }
    
    pub fn delete_entry(&mut self, identity_token_id: Vec<u8>) 
    { 
        //self.entries.retain(|x| x.identification_token_id.cmp(&identity_token_id) != Ordering::Equal);
        self.update_storage();
    }

    fn update_storage(&self)
    {
        let writer = File::create(self.file_name).unwrap();
        let _ = serde_json::to_writer_pretty(writer, &self.entries);
    }

    fn load_data(&mut self)
    {
        let reader = File::open(self.file_name);
        if let Ok(file) = reader
        {
            self.data =  serde_json::from_reader(file).unwrap_or_else(|_| Vec::new())
        }
    }
}

