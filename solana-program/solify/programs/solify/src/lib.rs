use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod state;
pub mod error;
pub mod types;
pub mod analyzer;

#[cfg(test)]
mod tests;

pub use types::IdlData;

declare_id!("4ZccwG28ne8hTmKLWDyDZmHw35su99iUxFRj5jy1p1Cb");

#[program]
pub mod solify {

    use super::*;

    pub fn store_idl_data(ctx: Context<StoreIdl>, idl_data: IdlData, program_id: Pubkey) -> Result<()> {
        ctx.accounts.store_idl(idl_data, program_id)
    }

    pub fn update_idl_data(ctx: Context<UpdateIdl>, idl_data: IdlData, program_id: Pubkey) -> Result<()> {
        let _ = program_id; // Used in seeds constraint
        ctx.accounts.update_idl(idl_data)
    }

    pub fn generate_metadata(
        ctx: Context<GenerateMetadata>, 
        execution_order: Vec<String>,
        program_id: Pubkey, 
        program_name: String,
        paraphrase: String,
    ) -> Result<()> {
        ctx.accounts.generate_metadata(execution_order, program_id, program_name, paraphrase)
    }

}

