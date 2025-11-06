use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod state;
pub mod error;
pub mod events;
pub mod types;
pub mod analyzer;
pub mod constants;

pub use types::IdlData;

declare_id!("67GqHdXxaRL3SYuRn29tzbRjMJCbNxaCAyaZpKNXu76b");

#[program]
pub mod solify {

    use super::*;

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        ctx.accounts.initialize_user()
    }

    pub fn generate_metadata(
        ctx: Context<GenerateMetadata>, 
        idl_data: IdlData,
        execution_order: Vec<String>, 
        program_id: Pubkey, 
        program_name: String
    ) -> Result<()> {
        ctx.accounts.generate_metadata(idl_data, execution_order, program_id, program_name)
    }
}

