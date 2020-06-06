/*

This module contains the basic data for generic ACM modules

*/

use std::fmt;

pub mod generic_whitelist;

#[derive(Clone)]
pub struct WhitelistAccessRequest
{
    pub identity_token_number: Vec<u8>,
    pub access_point_id: u32
}

impl fmt::Display for WhitelistAccessRequest
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}: {:?}", self.access_point_id, self.identity_token_number)
    }
}