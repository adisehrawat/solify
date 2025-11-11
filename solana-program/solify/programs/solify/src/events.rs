use anchor_lang::prelude::*;

#[event]
pub struct UserProfileCreated {
    pub user: Pubkey,
    pub timestamp: i64,
}
