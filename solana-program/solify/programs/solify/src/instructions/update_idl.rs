use anchor_lang::prelude::*;
use crate::state::IdlStorage;
use crate::types::IdlData;

#[derive(Accounts)]
#[instruction(idl_data: IdlData, program_id: Pubkey)]
pub struct UpdateIdl<'info> {
    #[account(
        mut,
        realloc = 8 + 32 + 32 + 8 + idl_data.try_to_vec().unwrap().len(),
        realloc::payer = authority,
        realloc::zero = false,
        seeds = [b"idl_storage", program_id.as_ref(), authority.key().as_ref()],
        bump
    )]
    pub idl_storage: Account<'info, IdlStorage>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateIdl<'info> {
    pub fn update_idl(
        &mut self,
        idl_data: IdlData,
    ) -> Result<()> {
        let clock = Clock::get()?;
        self.idl_storage.update(idl_data, clock.unix_timestamp)?;

        Ok(())
    }
}
