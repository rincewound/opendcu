

pub struct WhitelistEntry;

pub trait WhitelistEntryProvider
{
    fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<WhitelistEntry>;
    fn put_entry(&self,entry: WhitelistEntry);
}


pub struct SqliteEntryProvider;

impl WhitelistEntryProvider for SqliteEntryProvider
{
    fn get_entry(&self, identity_token_id: Vec<u8>) -> Option<WhitelistEntry>
    {
        None
    }

    fn put_entry(&self, entry: WhitelistEntry)
    {

    }
}