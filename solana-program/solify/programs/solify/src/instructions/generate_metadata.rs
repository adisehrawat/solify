use anchor_lang::prelude::*;
use crate::{
    analyzer::DependencyAnalyzer, 
    error::SolifyError, 
    state::{IdlStorage, ProgramTestHistory, TestMetadataConfig, user_config::UserConfig},
    types::IdlData,
};

#[derive(Accounts)]
#[instruction(execution_order: Vec<String>, program_id: Pubkey)]
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
    /// CHECK: We manually deserialize only the needed fields to avoid heap overflow
    #[account(mut)]
    pub idl_storage: UncheckedAccount<'info>,
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

        let idl_storage_data = self.idl_storage.try_borrow_data()?;
        
        let discriminator = &idl_storage_data[0..8];
        let expected_discriminator = IdlStorage::DISCRIMINATOR;
        require!(
            discriminator == expected_discriminator,
            SolifyError::InvalidAccountData
        );
        
        let mut data_slice = &idl_storage_data[8..];
        let authority = Pubkey::deserialize(&mut data_slice)?;
        let stored_program_id = Pubkey::deserialize(&mut data_slice)?;
        
        require!(
            authority == self.authority.key(),
            SolifyError::Unauthorized
        );
        require!(
            stored_program_id == program_id,
            SolifyError::InvalidProgramId
        );
        
        let idl_data = IdlData::deserialize(&mut data_slice)?;
        
        let analyzer = DependencyAnalyzer::new();
        let test_metadata = analyzer.analyze_dependencies(&idl_data, &execution_order, program_id.to_string())?;

        let program_history = ProgramTestHistory {
            program_id: program_id.to_string(),
            program_name: program_name.clone(),
            test_count: test_metadata.test_cases.iter().map(|test_case| test_case.positive_cases.len() as u32 + test_case.negative_cases.len() as u32).sum(),
            last_generated_at: clock.unix_timestamp
        };

        self.user_config.update_after_generation(program_history, clock.unix_timestamp);

        self.test_metadata_config.initialize(
            self.authority.key(), 
            program_id, 
            program_name, 
            test_metadata, 
            clock.unix_timestamp
        )?;
        
        Ok(())
    }
}