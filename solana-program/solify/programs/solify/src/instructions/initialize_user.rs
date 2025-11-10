use anchor_lang::prelude::*;

use crate::{ events::UserProfileCreated, state::user_config::UserConfig };

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + UserConfig::INIT_SPACE,
        seeds = [b"user_config", authority.key().as_ref()],
        bump
    )]
    pub user_config: Account<'info, UserConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeUser<'info> {
    pub fn initialize_user(&mut self, bumps: &InitializeUserBumps) -> Result<()> {
        let clock = Clock::get()?;

        self.user_config.initialize(self.authority.key(), bumps.user_config, clock.unix_timestamp);
        emit!(UserProfileCreated {
            user: self.authority.key(),
            timestamp: clock.unix_timestamp,
        });
        msg!("User profile created for: {}", self.authority.key());
        Ok(())
    }
}
