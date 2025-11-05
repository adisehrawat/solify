
use anchor_lang::prelude::*;

// use crate::error::SolifyError;
use crate::state::user_config::UserConfig;

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
    pub fn initialize_user(&mut self) -> Result<()> {
        self.user_config.set_inner(UserConfig {
            authority: self.authority.key(),
            total_tests_generated: 0,
            programs_tested: vec![],
            created_at: Clock::get()?.unix_timestamp,
            last_generated_at: 0,
            bump: self.user_config.bump,
        });
        Ok(())
    }
}