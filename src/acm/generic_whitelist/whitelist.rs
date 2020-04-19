
pub enum AccessFlags
{
    Access,
    Pin
}

#[derive(Clone)]
pub struct TimePair
{
    from: u16,
    to: u16,
    access_flags: u16
}

#[derive(Clone)]
pub struct TimeProfile
{
    dayflags: u16,
    time_pairs: Vec<TimePair>
}

#[derive(Clone)]
pub struct AccessProfile
{
    // All access points, for which this profile is valid
    access_points: Vec<u32>,
    time_pro: TimeProfile
}

#[derive(Clone)]
pub struct WhitelistEntry
{
    pub access_token_id: Vec<u8>,
    pub access_profiles: Vec<AccessProfile> // Note: This should be a ref to another table or similar
}

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


