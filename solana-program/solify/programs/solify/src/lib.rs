use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod state;
pub mod error;
pub mod events;
pub mod types;
pub mod analyzer;

pub use types::IdlData;

declare_id!("67GqHdXxaRL3SYuRn29tzbRjMJCbNxaCAyaZpKNXu76b");

#[program]
pub mod solify {

    use super::*;

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        ctx.accounts.initialize_user(&ctx.bumps)
    }
    pub fn store_idl_data(ctx: Context<StoreIdl>, idl_data: IdlData, program_id: Pubkey) -> Result<()> {
        ctx.accounts.store_idl(idl_data, program_id)
    }

    pub fn generate_metadata(
        ctx: Context<GenerateMetadata>, 
        execution_order: Vec<String>,
        program_id: Pubkey, 
        program_name: String,
    ) -> Result<()> {
        ctx.accounts.generate_metadata( execution_order, program_id, program_name)
    }
}

