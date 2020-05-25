use serde::{Serialize};
use serde::de::DeserializeOwned;
use std::{fs::File};

pub trait ObjectStorage<T>
{
    fn get_entry<P>(&self, filter: P) -> Option<T> where P: FnMut(&T) -> bool;
    fn put_entry(&mut self, entry: T);
    fn delete_entry<P>(&mut self, filter: P)where P: FnMut(&T) -> bool;
    fn update_storage(&self);
}

pub struct json_storage<ValueType>
{
    data: Vec<ValueType>,
    file_name: String
}

impl <ValueType> json_storage<ValueType> where
    ValueType: Clone + Serialize + DeserializeOwned
{
    pub fn new(file_name: String) -> Self
    {
        let mut ret_val = json_storage
        {
            data: Vec::new(),
            file_name
        };

        ret_val.load_data();
        return ret_val;
    }

    fn load_data(&mut self)
    {
        let reader = File::open(self.file_name.as_str());
        if let Ok(file) = reader
        {
            self.data =  serde_json::from_reader(file).unwrap_or_else(|_| Vec::new())
        }
    }
}

impl <ValueType> ObjectStorage<ValueType> for json_storage<ValueType> where
    ValueType: Clone + Serialize + DeserializeOwned
{
    fn get_entry<P>(&self, mut filter: P) -> Option<ValueType>  where
        P: FnMut(&ValueType) -> bool
    {
        for item in self.data.iter()
        {
            if filter(item)
            {
                return Some(item.clone())
            }
        }
        None
    }

    fn put_entry(&mut self, entry: ValueType) 
    {         
        self.data.push(entry);
    }
    
    fn delete_entry<P>(&mut self, filter: P) where
        P: FnMut(&ValueType) -> bool
    { 
        self.data.retain(filter);
    }

    fn update_storage(&self)
    {
        let writer = File::create(self.file_name.as_str()).unwrap();
        let _ = serde_json::to_writer_pretty(writer, &self.data);
    }
}

