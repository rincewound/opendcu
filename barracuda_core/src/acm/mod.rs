/*

This module contains the basic data for generic ACM modules

*/

pub mod generic_whitelist;

#[derive(Clone)]
pub struct WhitelistAccessRequest
{
    pub identity_token_number: Vec<u8>,
    pub access_point_id: u32
}