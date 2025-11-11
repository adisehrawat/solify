use anchor_lang::prelude::*;

use crate::types::IdlData;

#[account]
#[derive(InitSpace, Debug)]
pub struct IdlStorage {
    pub authority: Pubkey,
    pub program_id: Pubkey,
    pub idl_data: IdlData,
    pub timestamp: i64, 
}

impl IdlStorage {
    
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        program_id: Pubkey,
        idl_data: IdlData,
        timestamp: i64,
    ) -> Result<()> {
        self.authority = authority;
        self.program_id = program_id;
        self.idl_data = idl_data;
        self.timestamp = timestamp;
        Ok(())
    }

    pub fn update(
        &mut self,
        idl_data: IdlData,
        timestamp: i64,
    ) -> Result<()> {
        self.idl_data = idl_data;
        self.timestamp = timestamp;
        Ok(())
    }
}

