use anchor_lang::prelude::*;
use crate::state::IdlStorage;
use crate::types::IdlData;

#[derive(Accounts)]
#[instruction(idl_data: IdlData, program_id: Pubkey)]
pub struct StoreIdl<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 4 + idl_data.try_to_vec().unwrap().len(),
        seeds = [b"idl_storage", program_id.as_ref(), authority.key().as_ref()],
        bump
    )]
    pub idl_storage: Account<'info, IdlStorage>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> StoreIdl<'info> {
    pub fn store_idl(
        &mut self,
        idl_data: IdlData,
        program_id: Pubkey,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        self.idl_storage.initialize(
            self.authority.key(),
            program_id,
            idl_data,
            clock.unix_timestamp,
        )?;
        
        Ok(())
    }
}
