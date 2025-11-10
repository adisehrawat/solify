use anchor_lang::prelude::*;
use crate::{
    analyzer::{DependencyAnalyzer}, 
    error::SolifyError, 
    state::{TestMetadataConfig, user_config::UserConfig, IdlStorage},
    types::IdlData,
};

#[derive(Accounts)]
#[instruction(idl_data: IdlData, execution_order: Vec<String>, program_id: Pubkey)]
pub struct GenerateMetadata<'info> {
    #[account(
        mut,
        seeds = [b"user_config", authority.key().as_ref()],
        bump = user_config.bump
    )]
    pub user_config: Account<'info, UserConfig>,
    #[account(
        init,
        payer = authority,
        space = TestMetadataConfig::DISCRIMINATOR.len() + TestMetadataConfig::INIT_SPACE, 
        seeds = [b"tests_metadata", program_id.as_ref(), authority.key().as_ref()],
        bump
    )]
    pub test_metadata_config: Account<'info, TestMetadataConfig>,
    #[account(mut, constraint = idl_storage.authority == authority.key() && idl_storage.program_id == program_id)]
    pub idl_storage: Account<'info, IdlStorage>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> GenerateMetadata<'info> {
    pub fn generate_metadata(
        &mut self,
        execution_order: Vec<String>,
        program_id: Pubkey,
        program_name: String,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        require!(
            !execution_order.is_empty(),
            SolifyError::InvalidInstructionOrder
        );

        // let idl_data = &self.idl_storage.idl_data;
        // let idl_data = IdlData::try_from_slice(&self.idl_storage.idl_data.try_to_vec().unwrap())?;
        
        // let analyzer = DependencyAnalyzer::new();
        // let test_metadata = analyzer.analyze_dependencies(&idl_data, &execution_order, program_id.to_string())?;


        self.user_config.update_after_generation(
            clock.unix_timestamp
        );

        // self.test_metadata_config.initialize(
        //     self.authority.key(), 
        //     program_id, 
        //     program_name, 
        //     test_metadata, 
        //     clock.unix_timestamp
        // )?;
        
        Ok(())
    }
}