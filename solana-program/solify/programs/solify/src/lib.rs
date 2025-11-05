use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod state;
pub mod error;

declare_id!("67GqHdXxaRL3SYuRn29tzbRjMJCbNxaCAyaZpKNXu76b");

#[program]
pub mod solify {
    use super::*;

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        ctx.accounts.initialize_user()
    }
}

