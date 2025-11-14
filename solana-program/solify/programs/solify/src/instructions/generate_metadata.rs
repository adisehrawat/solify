use anchor_lang::prelude::*;
use crate::{
    analyzer::DependencyAnalyzer,
    error::SolifyError,
    state::{ IdlStorage, TestMetadataConfig },
    types::IdlData,
};

#[derive(Accounts)]
#[instruction(execution_order: Vec<String>, program_id: Pubkey, program_name: String, paraphrase: String)]
pub struct GenerateMetadata<'info> {
    #[account(
        init,
        payer = authority,
        space = TestMetadataConfig::DISCRIMINATOR.len() + TestMetadataConfig::INIT_SPACE,
        seeds = [
            b"tests_metadata",
            program_id.as_ref(),
            authority.key().as_ref(),
            paraphrase.as_bytes(),
        ],
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
        paraphrase: String
    ) -> Result<()> {
        let clock = Clock::get()?;

        let idl_storage_data = self.idl_storage.try_borrow_data()?;
        
        require!(
            idl_storage_data.len() >= 8,
            SolifyError::InvalidAccountData
        );

        let discriminator = &idl_storage_data[0..8];
        let expected_discriminator = IdlStorage::DISCRIMINATOR;
        require!(discriminator == expected_discriminator, SolifyError::InvalidAccountData);

        let mut data_slice = &idl_storage_data[8..];
        let authority = Pubkey::deserialize(&mut data_slice)?;
        let stored_program_id = Pubkey::deserialize(&mut data_slice)?;

        require!(authority == self.authority.key(), SolifyError::Unauthorized);
        require!(stored_program_id == program_id, SolifyError::InvalidProgramId);

        let idl_data = IdlData::deserialize(&mut data_slice)?;

        let analyzer = DependencyAnalyzer::new();
        let test_metadata = analyzer.analyze_dependencies(
            &idl_data,
            &execution_order,
            program_id.to_string()
        )?;

        self.test_metadata_config.initialize(
            self.authority.key(),
            program_id,
            program_name,
            paraphrase,
            test_metadata,
            clock.unix_timestamp
        )?;

        Ok(())
    }
}
